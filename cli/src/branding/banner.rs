use super::palette::Palette;

pub fn print_banner(palette: &Palette) {
    if !palette.enabled {
        return;
    }

    let art = r#"
       /\   /\
      /  \ /  \
     /    V    \
    /  .--^--.  \
   /  /  @ @  \  \
  /  |  \___/  |  \
 /    \_______/    \
/__________________\
"#;

    println!("{}", palette.brand().apply_to(art.trim_end()));
    println!(
        "{}",
        palette
            .brand()
            .apply_to("  SolHawk — Solana Circuit Breaker Protocol")
    );
    println!();
}
