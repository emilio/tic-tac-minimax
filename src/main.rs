mod minimax;
mod state;

use minimax::MiniMaxTree;
use state::CheckBox;

fn main() {
    let mut tree = MiniMaxTree::new(CheckBox::X);

    tree.update_to_depth(::std::usize::MAX);
    println!("{}", tree.depth());

    let mut dump = String::new();
    tree.dump(&mut dump).unwrap();
    print!("{}", dump);
}
