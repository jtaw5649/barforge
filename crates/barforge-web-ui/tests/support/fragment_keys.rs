use dioxus::core::{DynamicNode, VNode};

pub fn find_fragment_keys(root: &VNode, expected_len: usize) -> Option<Vec<Option<String>>> {
    let mut fragments = Vec::new();
    collect_fragment_keys(root, &mut fragments);
    fragments
        .into_iter()
        .find(|keys| keys.len() == expected_len)
}

fn collect_fragment_keys(node: &VNode, fragments: &mut Vec<Vec<Option<String>>>) {
    for dynamic in node.dynamic_nodes.iter() {
        if let DynamicNode::Fragment(children) = dynamic {
            let keys = children.iter().map(|child| child.key.clone()).collect();
            fragments.push(keys);
            for child in children {
                collect_fragment_keys(child, fragments);
            }
        }
    }
}
