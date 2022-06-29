use crate::{console::Green, print, println};

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        print!("{}... \t", core::any::type_name::<T>());
        self();
        println!("{}", Green("[ok]"));
    }
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Testable]) {
    use crate::qemu::exit_success;

    println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }

    exit_success();
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
