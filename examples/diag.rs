//  DIAG.rs
//    by Lut99
//
//  Description:
//!   Showcase of pretty rendered diagnostics.
//

use ast_toolkit2::diag::layout4::renderers::HexRenderer;
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
    let id = sources.add_bytes("tragedy.txt", b"Did you ever hear the tragedy of Darth Plagueis The Wise? I thought not. It's not a story the Jedi would tell you. It's a Sith legend. Darth Plagueis was a Dark Lord of the Sith, so powerful and so wise he could use the Force to influence the midichlorians to create life... He had such a knowledge of the dark side that he could even keep the ones he cared about from dying. The dark side of the Force is a pathway to many abilities some consider to be unnatural. He became so powerful... the only thing he was afraid of was losing his power, which eventually, of course, he did. Unfortunately, he taught his apprentice everything he knew, then his apprentice killed him in his sleep. Ironic. He could save others from death, but not himself.");
    let renderer = if args.binary { HexRenderer::new(100, &sources) } else { todo!() };

    // Sourceless error
    let diag = Diag::error("You've attempted to listen to the tragedy of Darth Plagueis The Wise as a Jedi").with_theme(Theme::RUST);
    println!("{}", diag.display(&renderer));

    // Error with source
    let diag = Diag::error("Sith lord detected")
        .with_theme(Theme::RUST)
        .with_annot(Annotation::error(Loc::encapsulate_range(id, 33..56)).with_msg("He's here"));
    println!("{}", diag.display(&renderer));

    // Error with help
    let diag = Diag::warning("It seems like the Sith Lord loses his power")
        .with_theme(Theme::RUST)
        .with_annot(Annotation::warning(Loc::encapsulate_range(id, 522..542)))
        .with_annot(Annotation::help(Loc::encapsulate_range(id, 573..579)).with_msg("This further solidifies that idea"));
    println!("{}", diag.display(&renderer));
}
