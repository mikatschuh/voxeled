fn func() {
    let x = Number::new(0);
    // x = Number(vec![0])

    x.add(Number::new(1));
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
