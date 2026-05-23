# wurl.nvim

A Neovim plugin providing regex-based syntax highlighting and filetype detection for Wurl (`.wurl`) files. It matches the design of the official VS Code extension.

## Installation

### [lazy.nvim](https://github.com/folke/lazy.nvim)

Add the following to your `lazy.nvim` configuration:

```lua
{
  "your-org/wurl.nvim", -- Replace with actual GitHub repository URL
  name = "wurl",
  ft = "wurl",
  opts = {}, -- Calls the `setup` function automatically
}
```

### [packer.nvim](https://github.com/wbthomason/packer.nvim)

```lua
use {
  "your-org/wurl.nvim",
  config = function()
    require("wurl").setup()
  end
}
```

## Features

- Native Neovim `filetype.lua` detection for `.wurl` files.
- Regex-based syntax highlighting for keywords, HTTP methods, targets, and matchers matching the VS Code textmate grammar.
