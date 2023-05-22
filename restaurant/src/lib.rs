pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}


pub mod restaurant {
    pub mod hosting {
        pub fn add_to_waitlist() {
            println!("add_to_waitlist");
        }
    }
}
