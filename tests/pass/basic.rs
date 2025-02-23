use piggyback::piggyback;

fn ok_number() -> Result<i32, i32> {
    Ok(5)
}

#[piggyback]
fn main() -> Result<(), i32> {
    let mut x = 0;
    #[piggyback(|_: &i32| x += 5)]
    let _ = ok_number()?;
    assert_eq!(x, 0);
    Ok(())
}
