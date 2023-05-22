
use ferris_says::say;
use std::io::{stdout, BufWriter};
use std::io;
use rand::Rng;

use garden::flower::rose;
use restaurant::restaurant::hosting::add_to_waitlist;
use mymod::mymod::print as printHello;

pub mod mymod;


fn startWebServer(){

}


fn mainConsole() {
    println!("Hello, world!");
    let k = input(1, 2);


    println!("Hello, world! {}", k);

    //main2();

    let rect1 = Rectangle { width: 30, height: 50 };
    println!("rect1 is {}", rect1.width * rect1.height);
    let area = rect1.area();
    println!("rect1 is {}", area);


    println!("color red is {}", color::red as i32);
    println!("color blue is {}", color::blue as i32);


    rose();
    printHello();


    add_to_waitlist();
}


struct Rectangle {
    width: u32,
    height: u32,
}


impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }
}

fn input(x:i32, y:i32) -> i32 {
    println!("Please input your guess. {}, {}" ,x, y);
    return x+y
}

fn myfunction() {


    let secret_number = rand::thread_rng().gen_range(1..=100);
    println!("The secret number is: {}", secret_number);

    let stdout = stdout();
    let message = String::from("Hello fellow Rustaceans!");
    let width = message.chars().count();

    let mut writer = BufWriter::new(stdout.lock());
    say(message.as_bytes(), width, &mut writer).unwrap();


    let mut input  = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    println!("You guessed: {}", input);



    let _guess: u32 = "42".parse().expect("Not a number!");

}


enum color{
    red,
    green,
    blue,
}


mod garden{
    pub fn print_hello(){
        println!("Hello, world!");
    }
    pub mod flower{
        pub fn rose(){
            println!("rose");
        }
    }

}
