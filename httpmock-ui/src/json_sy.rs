
#[cfg(test)]
mod tests {
    use nom::bytes::complete::is_not;
    use nom::sequence::pair;
    use nom::{
        IResult,
        sequence::delimited,
        // see the "streaming/complete" paragraph lower for an explanation of these submodules
        character::complete::char,
      };
    use serde_json::Value;

 
    fn parens(input: &str) -> IResult<&str, &str> {
        delimited(char('('), is_not(")"), char(')'))(input)
    } 

    fn pair_parse(input:&str) -> IResult<&str,(char,char)> {
        pair(char('('),char(')'))(input)
    }

    #[test]
    fn test_parse_demo1() {
        let input = "(AD(DC(SD)DS)SC)";
        let x =parens(input);
        let (a,b) = x.unwrap();
        println!("{}=={}",a,b);
    }
    
    #[test]
    fn test_pretty() {
            // Some JSON input data as a &str. Maybe this comes from the user.
        let data = r#"
        { "name": "John Doe", "age": 43,
            "phones": [ "+44 1234567", "+44 2345678" ]
        }"#;

    // Parse the string of data into serde_json::Value.
        let v: Value = serde_json::from_str(data).unwrap();

        let s = serde_json::to_string_pretty(&v).unwrap();
    // Access parts of the data by indexing with square brackets.
        println!("{}", s);
    }

    #[test]
    fn test_fake() {

        use fake::{Fake, ResultFaker};
        use fake::faker::name::en::Name;

        // generate name on success but some error code on failure
        let f = ResultFaker::ok(Name());
        for _ in 0..2 {
            let a = f.fake::<Result<String, u8>>();
            dbg!(a);
        }
        let f = ResultFaker::with(3.., 1..10);
        for _ in 0..5 {
            let a = f.fake::<Result<u32, usize>>();
            dbg!(a);
        }
    }
}