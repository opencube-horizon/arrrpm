# Arrrpm

Simple standalone and portable binary to inspect given RPMs, generate a dependency tree in DOT format.

```console
$ ./arrrpm help
Usage: arrrpm <command> [<args>]

Some RPM tools

Options:
  --help            display usage information

Commands:
  dep-tree          Generate a dot graph with dependencies from a set of RPMs
  ls                List files in the given RPM

$ ./arrrpm help dep-tree
Usage: arrrpm dep-tree [<rpms...>] [-x <exclude...>]

Generate a dot graph with dependencies from a set of RPMs

Positional Arguments:
  rpms              list of RPMs

Options:
  -x, --exclude     pattern for names to exclude
  --help            display usage information

$ ./arrrpm help ls
Usage: arrrpm ls <rpm>

List files in the given RPM

Positional Arguments:
  rpm               RPM

Options:
  --help            display usage information
```

Example usage when using a terminal (`wezterm`) which can display images inline:

```console
$ ./arrrpm dep-tree -x '^(grep|coreutils|suse-kernel-rpm-scriptlets|python-rpm-macros)$' -x '64kb$' ~/Downloads/some-archive/*.rpm | dot -Tjpg -Grankdir=LR | wezterm imgcat
```
