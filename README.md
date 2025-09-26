# Arrrpm

Simple standalone and portable binary to inspect given RPMs, generate a dependency tree in DOT format.

```console
$ arrrpm help
Some RPM tools

Usage: arrrpm <COMMAND>

Commands:
  dep-tree  Generate a dot graph with dependencies from a set of RPMs
  ls        List files in the given RPM(s)
  info      List info for the given RPM
  cat       Cat content from the RPM (mainly scriptlets)
  extract   Extract files from the RPM
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Example usage when using a terminal (`wezterm`) which can display images inline:

```console
$ arrrpm dep-tree \
  -x '^(grep|coreutils|suse-kernel-rpm-scriptlets|python-rpm-macros)$' \
  ~/Downloads/some-archive/*.rpm | dot -Tjpg -Grankdir=LR | wezterm imgcat
```

## Why Arrrpm?

Pirates like to take packages which don't belong to them and with Arrrpm you can inspect RPMs on systems where there are usually no tools to do so, without resorting to `rpm2cpio`.
And getting an overview on which packages belong together is important for a pirate either.
