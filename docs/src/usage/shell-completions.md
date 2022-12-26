# Generate shell completions

## zsh

To load completions in your current shell session:

```sh
$ autoload bashcompinit; bashcompinit
$ source <(release-plz generate-completions)
```

To load completions for every new session, execute once:

```sh
$ release-plz generate-completions zsh > _release-plz
$ sudo mv _release-plz /usr/local/share/zsh/site-functions/
```
## bash

To load completions in your current shell session:

```sh
$ source <(release-plz generate-completions)
```

To load completions for every new session, execute once:

```sh
$ release-plz generate-completions bash > ~/.local/share/bash-completion/completions/release-plz
```
Note: package *bash-completion* is required for this to work.

## fish

To load completions in your current shell session:

```sh
$ release-plz generate-completions fish | source
```

To load completions for every new session, execute once:

```sh
$ release-plz generate-completions fish > $HOME/.config/fish/completions/release-plz.fish
```
