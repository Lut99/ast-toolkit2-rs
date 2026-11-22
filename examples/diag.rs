//  DIAG.rs
//    by Lut99
//
//  Description:
//!   Showcase of pretty rendered diagnostics.
//

use ast_toolkit2::diag::renderers::{HexRenderer, Renderer, Utf8Renderer};
use ast_toolkit2::diag::{Annotation, Diag, Theme};
use ast_toolkit2::loc::Loc;
use ast_toolkit2::sources::Sources;
use clap::Parser;


/***** ARGUMENTS *****/
#[derive(Parser)]
struct Args {
    /// Whether to show the source in a hexgrid or not.
    #[clap(short, long)]
    binary: bool,
}





/***** ENTRYPOINT *****/
fn main() {
    let args = Args::parse();

    let mut sources = Sources::new();
    let id = sources.add_bytes("tragedy.txt", b"Did you ever hear the tragedy of Darth Plagueis The Wise?\nI thought not.\nIt's not a story the Jedi would tell you.\nIt's a Sith legend.\nDarth Plagueis was a Dark Lord of the Sith,\nso powerful and so wise he could use the Force\nto influence the midichlorians to create life...\nHe had such a knowledge of the dark side\nthat he could even keep the ones he cared about from dying.\nThe dark side of the Force is a pathway to many abilities\nsome consider to be unnatural.\nHe became so powerful...\nthe only thing he was afraid of was losing his power,\nwhich eventually, of course, he did.\nUnfortunately, he taught his apprentice everything he knew,\nthen his apprentice killed him in his sleep.\nIronic.\nHe could save others from death,\nbut not himself.");
    let renderer: Box<dyn Renderer<u8>> =
        if args.binary { Box::new(HexRenderer::new(100, &sources)) } else { Box::new(Utf8Renderer::new(100, &sources)) };

    // Sourceless error
    let diag = Diag::error("You've attempted to listen to the tragedy of Darth Plagueis The Wise as a Jedi").with_theme(Theme::COLOR);
    println!("{}", diag.display(&renderer));

    // Error with source
    let diag = Diag::error("Sith lord detected")
        .with_theme(Theme::COLOR)
        .with_annot(Annotation::error(Loc::encapsulate_range(id, 33..56)).with_msg("He's here"));
    println!("{}", diag.display(&renderer));

    // Error with help
    let diag = Diag::warning("It seems like the Sith Lord loses his power")
        .with_theme(Theme::COLOR)
        .with_annot(Annotation::warning(Loc::encapsulate_range(id, 522..542)))
        .with_annot(Annotation::help(Loc::encapsulate_range(id, 573..579)).with_msg("This further solidifies that idea"));
    println!("{}", diag.display(&renderer));
}
