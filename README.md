
# Tiny-CMS
The tiniest CMS you can think of!
- Flat-file
- Markdown based (CommonMark compliant)
- Developer-first experience
- Cross-stack, cross programming, cross everything!

## How does this work?
Tiny CMS is a developer-first cms, based on simple markdown files and cli control.
Tiny CMS lets you create posts, edit them in markdown and have them fully available in html.

The CMS tool is not only a CLI tool for development, but rather can be an ultra tiny content delivery server, that can easily run beside any server, on any stack.

Tiny CMS is fully compatible with CommonMark as it uses [markdown-rs](https://github.com/wooorm/markdown-rs)


## Usage

```bash
cms init
cms new -n "My first post"
# edit your new post on "Content/src/My first post.md"
cms publish
# now content/publish has "My first post.html" 
```
