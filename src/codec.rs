use std::string::FromUtf8Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum EncodeError{
    TooLongTopic,
    TtlNotAvailable,
}

#[derive(Debug, Clone)]
pub enum DecodeError{
    TooShortPackage,
    TopicParsingError{err: FromUtf8Error},
}

impl fmt::Display for EncodeError{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self{
            EncodeError::TooLongTopic => {
                write!(f, "{}", "Topic is too long; Max topic length is 255 bytes")
            }
            EncodeError::TtlNotAvailable => {
                write!(f, "{}", "Ttl option available only if cash flag enable")
            }
        }
    }
}

impl fmt::Display for DecodeError{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self{
            DecodeError::TooShortPackage => {
                write!(f, "{}", "Row package is too short to decode")
            }
            DecodeError::TopicParsingError{err} => {
                std::fmt::Display::fmt(&err, f)
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Package{
    Subscribe{
        topic: String,
        priority: u8,
        is_subscribe: bool,
        silent_mod: bool
    },
    RegularMsg{
        topic: String,
        priority: u8,
        content: Vec<u8>,
        cash: bool,
        ttl: Option<u64>
    },
    ServiceMsg{
        topic: String,
        priority: u8,
        count: u64,
    }
}

impl Package{
    pub fn encode(self) -> Result<Vec<u8>, EncodeError>{
        let mut header: u8 = 0;
        match self{
            Package::Subscribe{
                topic,
                priority,
                is_subscribe,
                silent_mod
            } => {
                let topic_len = if topic.len() > u8::MAX as usize {
                    return Err(EncodeError::TooLongTopic)
                } else { topic.len() as u8 };
                if is_subscribe{
                    header |= 0b00100000;
                }
                if silent_mod{
                    header |= 0b00010000;
                }
                let mut result = vec![header, priority, topic_len];
                result.append(&mut topic.into_bytes());
                Ok(result)
            }
            Package::RegularMsg{
                topic,
                priority,
                mut content,
                cash,
                ttl
            } => {
                let topic_len = if topic.len() > u8::MAX as usize {
                    return Err(EncodeError::TooLongTopic)
                } else { topic.len() as u8 };
                header |= 0b11000000;
                if cash{
                    header |= 0b00001000;
                    if let Some(_) = ttl{
                        header |= 0b00000100;
                    }
                }else if let Some(_) = ttl {
                    return Err(EncodeError::TtlNotAvailable)
                }
                let mut result = vec![header, priority, topic_len];
                result.append(&mut topic.into_bytes());
                match ttl{
                    None => {
                        result.append(&mut content);
                    }
                    Some(ttl) => {
                        result.append(&mut ttl.to_be_bytes().to_vec());
                        result.append(&mut content);
                    }
                }
                Ok(result)
            }
            Package::ServiceMsg{
                topic,
                priority,
                count
            } => {
                let topic_len = if topic.len() > u8::MAX as usize {
                    return Err(EncodeError::TooLongTopic)
                } else { topic.len() as u8 };
                header |= 0b10000000;
                let mut result = vec![header, priority, topic_len];
                result.append(&mut topic.into_bytes());
                result.append(&mut count.to_be_bytes().to_vec());
                Ok(result)
            }
        }
    }
    pub fn decode(bytes: Vec<u8>) -> Result<Self, DecodeError>{
        if bytes.len() < 3{
            return Err(DecodeError::TooShortPackage)
        }
        let header = bytes[0];
        let priority = bytes[1];
        let topic_len = bytes[2];
        let topic = {
            let row = &bytes[3 .. 3+topic_len as usize];
            match String::from_utf8(row.to_owned()){
                Ok(topic) => {topic}
                Err(err) => { return Err(DecodeError::TopicParsingError{err}) }
            }
        };
        match header & 0b10000000 > 0{
            true => {
                //Msg
                match header & 0b01000000 > 0{
                    true => {
                        //Regular
                        let cash = header & 0b00001000 > 0;
                        let ttl_exist = header & 0b00000100 > 0;
                        match ttl_exist && cash {
                            true => {
                                let ttl = {
                                    let mut dst = [0u8; 8];
                                    dst.clone_from_slice(&bytes[3+topic_len as usize .. 11+topic_len as usize]);
                                    u64::from_be_bytes(dst)
                                };
                                let content = bytes[11+topic_len as usize .. bytes.len()].to_vec();
                                Ok(Package::RegularMsg{
                                    topic,
                                    priority,
                                    content,
                                    cash,
                                    ttl: Some(ttl)
                                })
                            }
                            false => {
                                let content = bytes[3+topic_len as usize .. bytes.len()].to_vec();
                                Ok(Package::RegularMsg{
                                    topic,
                                    priority,
                                    content,
                                    cash,
                                    ttl: None
                                })
                            }
                        }
                    }
                    false => {
                        //Service
                        let count = {
                            let mut dst = [0u8; 8];
                            dst.clone_from_slice(&bytes[3+topic_len as usize .. 11+topic_len as usize]);
                            u64::from_be_bytes(dst)
                        };
                        Ok(Package::ServiceMsg{
                            topic,
                            priority,
                            count
                        })
                    }
                }
            }
            false => {
                //Sub
                let is_subscribe = header & 0b00100000 > 0;
                let silent_mod = header & 0b00010000 > 0;
                Ok(Package::Subscribe{
                    topic,
                    priority,
                    is_subscribe,
                    silent_mod
                })
            }
        }
    }
}

#[cfg(test)]
mod package_test{
    use crate::codec::Package;

    #[allow(dead_code)]
    fn check_codec_correct(package: Package, expect_encode_error: bool, expect_decode_error: bool) {
        match package.clone().encode() {
            Ok(bytes) => {
                match Package::decode(bytes) {
                    Ok(result_package) => {
                        assert_eq!(package, result_package)
                    }
                    Err(err) => {
                        if expect_decode_error{
                            assert!(true);
                        }else{
                            assert!(false, "Unexpected decode error: {}", err)
                        }
                    }
                }
            }
            Err(err) => {
                if expect_encode_error{
                    assert!(true);
                }else{
                    assert!(false, "Unexpected encode error: {}", err)
                }
            }
        }
    }

    #[test]
    fn subscription_test(){
        let package = Package::Subscribe{
            topic: "some.topic".to_string(),
            priority: 111,
            is_subscribe: true,
            silent_mod: true
        };
        check_codec_correct(package, false, false);
    }

    #[test]
    fn subscription_void_topic_test(){
        let package = Package::Subscribe{
            topic: "".to_string(),
            priority: 111,
            is_subscribe: true,
            silent_mod: true
        };
        check_codec_correct(package, false, false);
    }

    #[test]
    fn service_msg_test(){
        let package = Package::ServiceMsg{
            topic: "some.topic".to_string(),
            priority: 111,
            count: 222
        };
        check_codec_correct(package, false, false);
    }

    #[test]
    fn service_void_topic_msg_test(){
        let package = Package::ServiceMsg{
            topic: "".to_string(),
            priority: 111,
            count: 222
        };
        check_codec_correct(package, false, false);
    }

    #[test]
    fn regular_msg_test(){
        let package = Package::RegularMsg{
            topic: "some.topic".to_string(),
            priority: 111,
            content: vec![0, 1, 2, 3, 4, 5],
            cash: true,
            ttl: Some(1)
        };
        check_codec_correct(package, false, false);
    }

    #[test]
    fn regular_void_topic_msg_test(){
        let package = Package::RegularMsg{
            topic: "".to_string(),
            priority: 111,
            content: vec![0, 1, 2, 3, 4, 5],
            cash: true,
            ttl: Some(1)
        };
        check_codec_correct(package, false, false);
    }

    #[test]
    fn regular_void_ttl_msg_test(){
        let package = Package::RegularMsg{
            topic: "".to_string(),
            priority: 111,
            content: vec![0, 1, 2, 3, 4, 5],
            cash: true,
            ttl: None
        };
        check_codec_correct(package, false, false);
    }

    #[test]
    fn regular_void_content_and_ttl_msg_test(){
        let package = Package::RegularMsg{
            topic: "".to_string(),
            priority: 111,
            content: vec![],
            cash: true,
            ttl: None
        };
        check_codec_correct(package, false, false);
    }

    #[test]
    fn regular_invalid_ttl_using_msg_test(){
        let package = Package::RegularMsg{
            topic: "".to_string(),
            priority: 111,
            content: vec![],
            cash: false,
            ttl: Some(10)
        };
        check_codec_correct(package, true, false);
    }
}
