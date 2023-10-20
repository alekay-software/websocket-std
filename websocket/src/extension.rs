use std::collections::HashMap;

pub struct Extension {
    name: String,
    params: Vec<Parameter>
}

impl Extension {
    pub fn new(name: String, params: Vec<Parameter>) -> Self {
        Extension { name, params }
    } 
}

pub struct Parameter {
    name: String,
    args: Option<HashMap<String, Option<String>>> 
}

impl Parameter {
    pub fn new(name: String, args: Option<HashMap<String, Option<String>>>) -> Self {
        Parameter { name, args }
    }

    pub fn print(&self) {
        println!("Name: {}", self.name);
        println!("--- Arguments ---");
        match &self.args {
            Some(arguments) => {
                for (k, v) in arguments {
                    let mut value = String::new();
                    if let Some(arg_val) = v {
                        value.push_str(arg_val);
                        print!("");
                    }
                    println!("\t- {k} -> {value}");
                }
            }, 
            None => println!("No arguments")
        }
    }
}

#[macro_export]
macro_rules! parameter {
    ($n:expr) => { Parameter::new($n.to_string(), None) };
    ($n:expr; $($arg:expr),*) => {
        {
            let mut args: HashMap<String, Option<String>> = HashMap::new();
            
            $(
                let arg_val = String::from($arg);
                let index = arg_val.find('=');

                match index { 
                    Some(i) => {
                        let arg = (&arg_val[0..i]).trim().to_string(); 
                        let val = (&arg_val[i+1..arg_val.len()]).trim().to_string();
                        args.insert(arg, Some(val));
                    },
                    None => { args.insert(arg_val, None); }
                }
            )* 
            Parameter::new($n.to_string(), Some(args))
        }
    };
}