# wurl.vim

A Vim plugin providing regex-based syntax highlighting and filetype detection for Wurl (`.wurl`) files. It matches the design of the official VS Code extension.

## Installation

### [vim-plug](https://github.com/junegunn/vim-plug)

Add this to your `.vimrc`:

```vim
Plug 'your-org/wurl.vim' " Replace with actual GitHub repository URL
```

### [Vundle](https://github.com/VundleVim/Vundle.vim)

```vim
Plugin 'your-org/wurl.vim'
```

### Native Vim 8+ Packages

```bash
git clone https://github.com/your-org/wurl.vim.git ~/.vim/pack/plugins/start/wurl.vim
```

## Features

- Native `ftdetect` for `.wurl` files.
- Regex-based syntax highlighting matching the VS Code textmate grammar.
