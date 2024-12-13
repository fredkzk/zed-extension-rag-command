# Rag Slash Command Extension

This Zed extension lets you query a local vector database with the slash command `/rag`. The built-in vector database and full-text search engine is powered by [aichat](https://github.com/sigoden/aichat/wiki/RAG-Guide).

**How to use it:**
- Implement aichat RAG on your documents
- Run the [aichat server](https://github.com/sigoden/aichat/wiki/Command-Line-Guide#run-server) to make the vectorized data (a yaml file located at C:\Users\...\AppData\Roaming\aichat\rags in Windows) available via http requests
- Clone this repo
- Adjust [this field in the zed_rag.rs file](https://github.com/fredkzk/zed-extension-rag-command/blob/4fd216267cb7e90e481012d332b40831fe3ff51e/src/zed_rag.rs#L60) to the name of your yaml file
- Adjust the [system prompt here](https://github.com/fredkzk/zed-extension-rag-command/blob/4fd216267cb7e90e481012d332b40831fe3ff51e/src/zed_rag.rs#L79)
- Open Zed with **Administrator privileges** (Windows)
- Open the Zed extensions panel ([guide](https://zed.dev/docs/extensions/installing-extensions)) (`ctrl-shift-x`) and select the whole folder of the extension. This will encode a wasm component for the extension, and compile it in Rust
- Use the slash command followed by your prompt from the Assistant Panel.

## Pre-requisites

[Install aichat](https://github.com/sigoden/aichat/blob/main/README.md).

## Setup

### Init RAG
**From Files:**
```
> Add documents: /tmp/dir1/file1;/tmp/dir1/file2
Loading /tmp/dir1/file1 [1/2]
Loading /tmp/dir1/file2 [2/2]
```

**From Directory:**
```
> Add documents: /tmp/dir1/
Load /tmp/dir1/ [1/1]
🚀 Loading file /tmp/dir1/file1
🚀 Loading file /tmp/dir1/file2
🚀 Loading file /tmp/dir1/file3
✨ Load directory completed
```

**From Directory (with file extensions filter):**
```
> Add documents: /tmp/dir2/**/*.{md,txt}
Load /tmp/dir2/**/*.{md,txt} [1/1]
🚀 Loading file /tmp/dir2/file2.md
🚀 Loading file /tmp/dir2/file1.txt
✨ Load directory completed
```

**From Url:**
```
> Add documents: https://sigoden.github.io/mynotes/tools/linux.html
Load https://sigoden.github.io/mynotes/tools/linux.html [1/1]
```

**From RecursiveUrl (websites):**
```
> Add documents: https://sigoden.github.io/mynotes/tools/**
Load https://sigoden.github.io/mynotes/tools/** [1/1]
⚙️  maxConnections=5 exclude='' extract='' toMarkdown=true
🚀 Crawling https://sigoden.github.io/mynotes/tools/
🚀 Crawling https://sigoden.github.io/mynotes/tools/docker.html
🚀 Crawling https://sigoden.github.io/mynotes/tools/git.html
🚀 Crawling https://sigoden.github.io/mynotes/tools/github-ci.html
🚀 Crawling https://sigoden.github.io/mynotes/tools/linux.html
🚀 Crawling https://sigoden.github.io/mynotes/tools/redis.html
✨ Crawl completed
```
> `**` is used to distinguish between Url and RecursiveUrl

### Custom Document Loaders
By default, AICHAT can only process text files. We need to configure the document_loaders so AICHAT can handle binary files such as PDFs and DOCXs.
```yaml
# Define document loaders to control how RAG and `.file`/`--file` load files of specific formats.
document_loaders:
  # You can add custom loaders using the following syntax:
  #   <file-extension>: <command-to-load-the-file>
  # Note: Use `$1` for input file and `$2` for output file. If `$2` is omitted, use stdout as output.
  pdf: 'pdftotext $1 -'                         # Load .pdf file, see https://poppler.freedesktop.org to set up pdftotext
  docx: 'pandoc --to plain $1'                  # Load .docx file, see https://pandoc.org to set up pandoc
```
The `document_loaders` configuration item is a map where the key represents the file extension and the value specifies the corresponding loader command.

To ensure the loaders function correctly, please verify that the required tools are installed.

### Use Reranker
AIChat RAG defaults to the `reciprocal_rank_fusion` algorithm for merging vector and keyword search results.

However, using a reranker to combine these results is a more established method that can yield greater relevance and accuracy.

You can add the following configuration to specify the default reranker.
```yaml
rag_reranker_model: null                    # Specifies the rerank model to use
```

You can also dynamically adjust the reranker using the `.set` command.

```
.set rag_reranker_model <tab>
```

### Run Server
AIChat comes with a built-in lightweight http server.

```
$ aichat --serve
Chat Completions API: http://127.0.0.1:8000/v1/chat/completions
Embeddings API:       http://127.0.0.1:8000/v1/embeddings
LLM Playground:       http://127.0.0.1:8000/playground
LLM Arena:            http://127.0.0.1:8000/arena?num=2
```

Change the listening address:
```
$ aichat --serve 0.0.0.0
$ aichat --serve 8080
$ aichat --serve 0.0.0.0:8080
```


## Usage
Select the `/rag` command from the assistant panel and type your prompt after it, for example:
```
/rag How do I install Deno?
```
