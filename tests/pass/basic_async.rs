use piggyback::piggyback;

async fn ok_number() -> Result<i32, i32> {
    Ok(5)
}

#[piggyback]
async fn func() -> Result<(), i32>{
    let mut x = 0;
    #[piggyback(|_: &i32| x += 5)]
    let _ = ok_number().await;
    Ok(())
}

fn main() {
}
