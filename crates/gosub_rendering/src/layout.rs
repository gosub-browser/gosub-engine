use taffy::prelude::*;

use gosub_html5::node::NodeId as GosubID;
use gosub_styling::render_tree::RenderTree;

use crate::style::get_style_from_node;

pub fn generate_taffy_tree(rt: &RenderTree) -> anyhow::Result<(TaffyTree, NodeId)> {
    let mut tree: TaffyTree<()> = TaffyTree::with_capacity(rt.nodes.len());

    rt.get_root();

    let root = add_children_to_tree(rt, &mut tree, rt.root)?;

    Ok((tree, root))
}

fn add_children_to_tree(
    rt: &RenderTree,
    tree: &mut TaffyTree<()>,
    node_id: GosubID,
) -> anyhow::Result<NodeId> {
    let Some(node) = rt.get_node(node_id) else {
        return Err(anyhow::anyhow!("Node not found"));
    };

    let mut children = Vec::with_capacity(node.children.len());

    for child in &node.children {
        let Ok(node) = add_children_to_tree(rt, tree, *child) else {
            continue;
        };

        children.push(node);
    }

    let style = get_style_from_node(node);

    tree.new_with_children(style, &children)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}
