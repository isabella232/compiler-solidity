pub struct Comment;

///
/// Skips all lexemes until `*/` is found.
///
impl Comment {
    pub fn parse<'a, I>(iter: &mut I)
    where
        I: crate::PeekableIterator<Item = &'a String>,
    {
        let mut elem = iter.next();
        while elem != None {
            if elem.unwrap() == "*/" {
                break;
            }
            elem = iter.next();
        }
        if elem == None {
            panic!("Can't find matching '*/', Yul input is ill-formed");
        }
    }
}
