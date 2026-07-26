_leaf() {
    local cur prev
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    case "$prev" in
        --theme)
            COMPREPLY=($(compgen -W "arctic forest ocean-dark solarized-dark" -- "$cur"))
            return
            ;;
        --editor|-e)
            COMPREPLY=($(compgen -W "nano vim vi nvim micro hx emacs jed code codium subl gedit kate mousepad zed xjed notepad notepad++" -- "$cur"))
            return
            ;;
        --inline)
            COMPREPLY=($(compgen -W "ansi plain" -- "$cur"))
            return
            ;;
        --config)
            COMPREPLY=($(compgen -W "reset" -- "$cur"))
            return
            ;;
        --auto-complete)
            COMPREPLY=($(compgen -W "bash zsh fish powershell" -- "$cur"))
            return
            ;;
        --show-keymap-actions)
            COMPREPLY=($(compgen -W "global viewer" -- "$cur"))
            return
            ;;
    esac

    if [[ "$cur" == -* ]]; then
        COMPREPLY=($(compgen -W "--help --version --watch --theme --editor --inline --width --picker --config --update --auto-complete --show-keymap-actions --include-hidden-keymap-actions -h -V -w -e" -- "$cur"))
        return
    fi

    COMPREPLY=($(compgen -f -- "$cur"))
}

complete -o filenames -F _leaf leaf
