use colored::Colorize;

/// Format a price value appropriately based on magnitude
pub fn fmt_price(p: f64) -> String {
    if p == 0.0 {
        return "0.00".into();
    }
    if p < 0.000001 {
        format!("{:.10}", p)
    } else if p < 0.001 {
        format!("{:.8}", p)
    } else if p < 1.0 {
        format!("{:.6}", p)
    } else if p < 100.0 {
        format!("{:.4}", p)
    } else {
        format!("{:.2}", p)
    }
}

/// Format a percentage change with color
pub fn fmt_change(pct: &str) -> String {
    let v: f64 = pct.parse().unwrap_or(0.0);
    if v > 0.0 {
        format!("+{:.2}%", v).green().to_string()
    } else if v < 0.0 {
        format!("{:.2}%", v).red().to_string()
    } else {
        format!("{:.2}%", v).white().to_string()
    }
}

/// Print the Binance Claw ASCII banner
pub fn print_banner() {
    println!(
        "{}",
        r#"
 ██████╗██╗      █████╗ ██╗    ██╗
██╔════╝██║     ██╔══██╗██║    ██║
██║     ██║     ███████║██║ █╗ ██║
██║     ██║     ██╔══██║██║███╗██║
╚██████╗███████╗██║  ██║╚███╔███╔╝
 ╚═════╝╚══════╝╚═╝  ╚═╝ ╚══╝╚══╝
"#
        .yellow()
        .bold()
    );
    println!(
        "  {} — Binance Price Sniper & OpenClaw Skill\n",
        format!("v{}", env!("CARGO_PKG_VERSION")).cyan()
    );
}

/// Format a volume number with K/M/B suffix
pub fn fmt_volume(v: &str) -> String {
    let val: f64 = v.parse().unwrap_or(0.0);
    if val >= 1_000_000_000.0 {
        format!("{:.2}B", val / 1_000_000_000.0)
    } else if val >= 1_000_000.0 {
        format!("{:.2}M", val / 1_000_000.0)
    } else if val >= 1_000.0 {
        format!("{:.2}K", val / 1_000.0)
    } else {
        format!("{:.4}", val)
    }
}
