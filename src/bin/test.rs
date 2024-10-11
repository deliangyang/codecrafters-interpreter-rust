struct Test {
    a: usize,
}

struct Container<'a> {
    test: Test,
    ref_test: Option<&'a Test>,
}

impl<'a> Container<'a> {
    fn new() -> Self {
        Container {
            test: Test { a: 1 },
            ref_test: None,
        }
    }

    fn set_ref(&'a mut self) {
        self.ref_test = Some(&self.test);
    }

    fn get_ref(&self) -> Option<&'a Test> {
        self.ref_test
    }
}

fn main() {
    let mut container = Container::new();
    container.set_ref();

    if let Some(test_ref) = container.get_ref() {
        println!("Value in Test: {}", test_ref.a);
    } else {
        println!("No reference found");
    }
}
