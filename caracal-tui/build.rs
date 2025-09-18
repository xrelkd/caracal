fn main() -> shadow_rs::SdResult<()> {
    drop(shadow_rs::ShadowBuilder::builder().build()?);
    Ok(())
}
