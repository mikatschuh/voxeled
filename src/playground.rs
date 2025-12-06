fn func() {
    let mut vector = vec![1, 2, 3];
    vector.push(4);
    println!("{:?}", vector); // [1, 2, 3, 4]
}

struct Number(Vec<u64>);
impl Number {
    fn new(val: u64) -> Self {
        Number(vec![val])
    }
    fn add(self, other: Self) -> Self {
        for (i, val) in self.0.into_iter().rev().enumerate() {}

        todo!()
    }
}
/*
         |¯¯¯¯¯¯¯¯¯¯|
Input -> | Funktion | -> Ouput
         |__________|

word = 10

vs.

word = •-.
         |
         '- 10
*/
