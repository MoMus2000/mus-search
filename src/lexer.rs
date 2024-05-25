#[derive(Debug)]
pub struct Lexer<'a>{
    content: &'a [char]
}

impl <'a> Lexer<'a>{
    pub fn new(content: &'a [char]) -> Self{
        Self { content }
    }

    fn trim_left(&mut self){
        while self.content.len() > 0 && self.content[0].is_whitespace(){
            self.content = &self.content[1..]
        }
    }

    fn next_token(&mut self) -> Option<String>{
        self.trim_left();

        if self.content.len() == 0 {
            return None
        }

        if self.content[0].is_numeric(){
            let mut n = 0;
            while n < self.content.len() && self.content[n].is_numeric(){
                n += 1;
            }
            let token = &self.content[0..n];
            self.content = &self.content[n..];
            return Some(token.iter().collect::<String>())
        }

        if self.content[0].is_alphabetic(){
            let mut n = 0;
            while n < self.content.len() && self.content[n].is_alphanumeric(){
                n += 1;
            }
            let token = &self.content[0..n];
            self.content = &self.content[n..];

            let tok = token.iter().collect::<String>();
            let mut env = crate::snowball::SnowballEnv::create(&tok);
            crate::snowball::algorithms::english_stemmer::stem(&mut env);
            let stemmed_term = env.get_current().to_string();

            return Some(stemmed_term)
        }
        let token = &self.content[0..1];
        self.content = &self.content[1..];
        return Some(token.iter().collect::<String>())
    }

}

impl<'a> Iterator for Lexer<'a>{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item>{
        self.next_token()
    }
}