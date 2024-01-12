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
$ ./arrrpm dep-tree \
  -x '^(grep|coreutils|suse-kernel-rpm-scriptlets|python-rpm-macros)$' \
  ~/Downloads/some-archive/*.rpm | dot -Tjpg -Grankdir=LR | wezterm imgcat
```

## Why Arrrpm?

Pirates like to take packages which don't belong to them and with Arrrpm you can inspect RPMs on systems where there are usually no tools to do so, without resorting to `rpm2cpio`.
And getting an overview on which packages belong together is important for a pirate either.
