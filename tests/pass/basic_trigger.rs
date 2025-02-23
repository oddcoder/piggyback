use piggyback::piggyback;

fn err_number() -> Result<i32, i32> {
    Err(5)
}

#[piggyback]
fn subroutine(x: &mut i32) -> Result<(), i32> {
    #[piggyback(|_: &i32| *x += 10)]
    let _ = err_number()?;
    assert_eq!(*x, 0);
    Ok(())
}

fn main() {
    let mut x = 0;
    let y = subroutine(&mut x);
    assert_eq!(x, 10);
    assert_eq!(y, Err(5));
}
