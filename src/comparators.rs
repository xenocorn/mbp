use std::cmp::min;

pub fn compare(a: &str, b: &str) -> bool{
    if a == "#" || b == "#" { return true }
    let a: Vec<&str> = a.split(".").collect();
    let b: Vec<&str> = b.split(".").collect();
    let count = min(a.len(), b.len());
    let def = a.len() == b.len();
    for i in 0 .. count{
        let (first, second) = (a[i], b[i]);
        if first == "#" || second == "#" {return true}
        if first == "*" || second == "*" {continue}
        if first != second {return false}
    }
    return def
}

#[cfg(test)]
mod compare_test{
    use crate::comparators::compare;

    #[test]
    fn match_1(){
        let a = "a.b.c.d.e";
        let b = "a.b.c.d.e";
        assert!(compare(a, b))
    }
    #[test]
    fn match_2(){
        let a = "a.b.*.d.*";
        let b = "a.*.c.d.*";
        assert!(compare(a, b))
    }
    #[test]
    fn match_3(){
        let a = "#";
        let b = "a.b.c.d.e";
        assert!(compare(a, b))
    }
    #[test]
    fn match_4(){
        let a = "#";
        let b = "#";
        assert!(compare(a, b))
    }
    #[test]
    fn match_5(){
        let a = "a.b.c.#";
        let b = "a.b.c.d.e";
        assert!(compare(a, b))
    }
    #[test]
    fn unmatch_1(){
        let a = "a.b.c.d.e";
        let b = "f.g.h.j.k";
        assert!(!compare(a, b))
    }
    #[test]
    fn unmatch_2(){
        let a = "a.b.c.d";
        let b = "a.b.c.d.e";
        assert!(!compare(a, b))
    }
    #[test]
    fn unmatch_3(){
        let a = "a.b.c.*";
        let b = "a.b.c.d.e";
        assert!(!compare(a, b))
    }
    #[test]
    fn unmatch_4(){
        let a = "a.b.c.d.*";
        let b = "a.b.c.d";
        assert!(!compare(a, b))
    }
}
