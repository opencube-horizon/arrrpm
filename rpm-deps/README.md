# rpm-deps

Simple standalone and portable binary to inspect given RPMs and generate a dependency tree in the DOT format.

```
```console
$ ./rpm-deps --help
Usage: rpm-deps [<rpms...>] [-x <exclude...>]

Generate a dot graph with dependencies from a set of RPMs

Positional Arguments:
  rpms              list of RPMs

Options:
  -x, --exclude     pattern for names to exclude
  --help            display usage information
```

Example usage when using a terminal (`wezterm`) which can display images inline:

```console
$ ./rpm-deps -x '^(grep|coreutils|suse-kernel-rpm-scriptlets|python-rpm-macros)$' -x '64kb$' ~/Downloads/some-archive/*.rpm | dot -Tjpg -Grankdir=LR | wezterm imgcat
```
