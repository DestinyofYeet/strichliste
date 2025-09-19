import sys

def parse(lines, indent=0):
    """Parse indented list into a nested tree structure"""
    tree = []
    while lines:
        line = lines[0]
        leading = len(line) - len(line.lstrip())
        if leading < indent:
            break
        if leading > indent:
            subtree, lines = parse(lines, leading)
            tree[-1][1] = subtree
        else:
            lines.pop(0)
            node = line.strip()[2:]  # remove "- "
            tree.append([node, []])
    return tree, lines

def normalize(tree):
    """Sort children recursively for comparison"""
    tree.sort(key=lambda x: x[0])
    for _, children in tree:
        normalize(children)
    return tree


def _diff_trees(tree1, tree2, name1 = "tree1", name2 = "tree2", path=""):
    """Recursively print differences between two trees"""
    names1 = {node: children for node, children in tree1}
    names2 = {node: children for node, children in tree2}

    for node in names1:
        if node not in names2:
            print(f"- Missing in {name1}: {path}{node}")
        else:
            diff_trees(names1[node], names2[node], name1, name2, path + node + "/")

    for node in names2:
        if node not in names1:
            print(f"- Missing in {name2}: {path}{node}")

def trees_equal(tree1, tree2):
    """Check deep equality of two normalized trees"""
    names1 = {node: children for node, children in tree1}
    names2 = {node: children for node, children in tree2}

    for node in names1:
        if node not in names2:
            return False
        else:
            trees_equal(names1[node], names2[node])

    for node in names2:
        if node not in names1:
            return False
    return True


def diff_trees(tree1, tree2, name1 = "tree1", name2 = "tree2", path=""):
    normalize(tree1)
    normalize(tree2)
    _diff_trees(tree1, tree2, name1, name2, path)

if __name__ == "__main__":
    # Example: read two trees separated by a blank line from stdin
    content = [line.rstrip("\n") for line in sys.stdin if line.strip() or line == "\n"]

    try:
        sep = content.index("")  # blank line separates two trees
    except ValueError:
        sys.exit("Please provide two trees separated by a blank line.")

    tree1_lines = content[:sep]
    tree2_lines = content[sep+1:]

    tree1, _ = parse(tree1_lines)
    tree2, _ = parse(tree2_lines)

    name1 = sys.argv[1] if len(sys.argv) > 1 else 'name1'
    name2 = sys.argv[2] if len(sys.argv) > 2 else 'name1'

    if trees_equal(tree1, tree2):
        exit(0)
    else:
        diff_trees(tree1, tree2, name1, name2)
        exit(1)
