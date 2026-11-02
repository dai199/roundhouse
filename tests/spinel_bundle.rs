//! The spinel-framework lane's shared builds (`emit_and_run::bundle`); a
//! no-op unless CI sets both directories.

#[path = "support/emit_and_run.rs"]
mod emit_and_run;

#[test]
#[ignore = "requires the Spinel toolchain, run in its CI lane"]
fn spinel_bundle_build() {
    let (Some(collected), Some(out)) =
        (std::env::var_os("RH_SPINEL_BUNDLE_COLLECT"), std::env::var_os("RH_SPINEL_BUNDLE_OUT"))
    else {
        return;
    };
    emit_and_run::bundle::build_all(std::path::Path::new(&collected), std::path::Path::new(&out));
}
