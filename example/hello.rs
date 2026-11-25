fn step_one(value: i32) -> i32 {
    let doubled_one = value * 2;
    doubled_one + 1
}

fn step_two(value: i32) -> i32 {
    let doubled_two = value * 2;
    doubled_two + 2
}

fn step_three(value: i32) -> i32 {
    let doubled_three = value * 2;
    doubled_three + 3
}

fn step_four(value: i32) -> i32 {
    let doubled_four = value * 2;
    doubled_four + 4
}

fn step_five(value: i32) -> i32 {
    let doubled_five = value * 2;
    doubled_five + 5
}

fn step_six(value: i32) -> i32 {
    let doubled_six = value * 2;
    doubled_six + 6
}

fn step_seven(value: i32) -> i32 {
    let doubled_seven = value * 2;
    doubled_seven + 7
}

fn step_eight(value: i32) -> i32 {
    let doubled_eight = value * 2;
    doubled_eight + 8
}

fn main() {
    let total = step_one(1)
        + step_two(2)
        + step_three(3)
        + step_four(4)
        + step_five(5)
        + step_six(6)
        + step_seven(7)
        + step_eight(8);
    println!("{total}");
}
