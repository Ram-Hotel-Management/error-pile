#[test]
fn testing_compilations() {
    use error_pile::ErrPile;
    let _a = ErrPile::custom("Some message");
    let _b = ErrPile::custom(format!("{} Some other error", "ErrCode:"));
}
