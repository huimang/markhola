    <script>
      window.MathJax = {
        startup: { typeset: false },
        svg: { fontCache: "none" }
      };
    </script>
    <script>__MERMAID_RUNTIME__</script>
    <script>__MATHJAX_RUNTIME__</script>
    <script>
      const status = document.getElementById("status");
      const findPanel = document.getElementById("findPanel");
      const findInput = document.getElementById("findInput");
      const findCount = document.getElementById("findCount");
      const replaceGroup = document.getElementById("replaceGroup");
      const replaceInput = document.getElementById("replaceInput");
      const findPrevious = document.getElementById("findPrevious");
      const findNext = document.getElementById("findNext");
      const replaceOne = document.getElementById("replaceOne");
      const replaceAll = document.getElementById("replaceAll");
      const findClose = document.getElementById("findClose");
      const emptyState = document.getElementById("emptyState");
      const previewPane = document.getElementById("previewPane");
      const editorPane = document.getElementById("editorPane");
      const tabsShell = document.getElementById("tabsShell");
      const tabsBar = document.getElementById("tabsBar");
      const tabNavigation = document.getElementById("tabNavigation");
      const previousTabs = document.getElementById("previousTabs");
      const nextTabs = document.getElementById("nextTabs");
      const newDocumentTab = document.getElementById("newDocumentTab");
      const editorLineNumbers = document.getElementById("editorLineNumbers");
      const editor = document.getElementById("editor");
      const content = document.getElementById("content");
      const workspace = document.querySelector(".workspace");
      const outlinePanel = document.getElementById("outlinePanel");
      const outlineClose = document.getElementById("outlineClose");
      const outlineList = document.getElementById("outlineList");
      const documentBase = document.getElementById("document-base");
      const aboutOverlay = document.getElementById("aboutOverlay");
      const aboutClose = document.getElementById("aboutClose");
      const aboutVersion = document.getElementById("aboutVersion");
      const aboutAuthor = document.getElementById("aboutAuthor");
      const aboutBuild = document.getElementById("aboutBuild");
      const aboutGithub = document.getElementById("aboutGithub");
      const aboutCopy = document.getElementById("aboutCopy");
      const emptyTitle = document.getElementById("emptyTitle");
      const emptyDescription = document.getElementById("emptyDescription");
      const outlineTitle = document.getElementById("outlineTitle");
      const aboutTitle = document.getElementById("aboutTitle");
      const aboutVersionLabel = document.getElementById("aboutVersionLabel");
      const aboutAuthorLabel = document.getElementById("aboutAuthorLabel");
      const aboutBuildLabel = document.getElementById("aboutBuildLabel");
      const aboutFooter = document.getElementById("aboutFooter");
      const appThemeStyle = document.getElementById("appThemeStyle");
      let appStrings = __APP_LANGUAGE__;
      const applyDocumentSize = (percent) => {
        const normalized = Math.min(200, Math.max(50, Number(percent) || 100));
        const scale = normalized / 100;
        const rootStyle = document.documentElement.style;
        rootStyle.setProperty("--document-font-size", `${17 * scale}px`);
        rootStyle.setProperty("--document-h1-font-size", `${36.8 * scale}px`);
        rootStyle.setProperty("--document-h2-font-size", `${27.52 * scale}px`);
        rootStyle.setProperty("--document-h3-font-size", `${21.6 * scale}px`);
        rootStyle.setProperty("--document-code-font-size", `${14 * scale}px`);
        rootStyle.setProperty("--editor-font-size", `${15 * scale}px`);
      };
      let mermaidInitialized = false;
      let mathJaxReadyPromise = null;
      let currentDocumentId = null;
      let currentDocumentMode = null;
      let findPanelVisible = false;
      let readonlyBaseHtml = "";
      let readonlyMatches = [];
      let readonlyActiveIndex = -1;
      let writableMatches = [];
      let outlineOpen = false;
      let writableActiveIndex = -1;

      const applyAppLanguage = (strings) => {
        appStrings = strings;
        document.documentElement.lang = strings.language;
        tabsBar.setAttribute("aria-label", strings.openDocuments);
        tabNavigation.setAttribute("aria-label", strings.moveThroughTabs);
        previousTabs.setAttribute("aria-label", strings.moveTabsBackward);
        nextTabs.setAttribute("aria-label", strings.moveTabsForward);
        newDocumentTab.setAttribute("aria-label", strings.newDocument);
        findInput.placeholder = strings.findInDocument;
        findInput.setAttribute("aria-label", strings.findInDocument);
        replaceInput.placeholder = strings.replaceWith;
        replaceInput.setAttribute("aria-label", strings.replaceWith);
        findPrevious.textContent = strings.previous;
        findNext.textContent = strings.next;
        replaceOne.textContent = strings.replace;
        replaceAll.textContent = strings.replaceAll;
        findClose.textContent = strings.close;
        emptyTitle.textContent = strings.noDocument;
        emptyDescription.textContent = strings.emptyDescription;
        editor.setAttribute("aria-label", strings.markdownEditor);
        outlinePanel.setAttribute("aria-label", strings.documentOutline);
        outlineTitle.textContent = strings.outline;
        outlineClose.setAttribute("aria-label", strings.closeOutline);
        outlineList.setAttribute("aria-label", strings.documentHeadings);
        aboutTitle.textContent = strings.about;
        aboutClose.setAttribute("aria-label", strings.closeAbout);
        aboutVersionLabel.textContent = strings.version;
        aboutAuthorLabel.textContent = strings.author;
        aboutBuildLabel.textContent = strings.build;
        aboutCopy.textContent = strings.copy;
        aboutFooter.textContent = strings.aboutFooter;
        for (const button of tabsBar.querySelectorAll("[data-close-document]")) {
          button.setAttribute(
            "aria-label",
            `${strings.closeDocument} ${button.dataset.fileName || ""}`.trim()
          );
        }
        updateFindCount(
          currentDocumentMode === "writable" ? writableMatches.length : readonlyMatches.length,
          currentDocumentMode === "writable" ? writableActiveIndex : readonlyActiveIndex
        );
        refreshOutline();
      };

      const hideAbout = () => {
        aboutOverlay.classList.add("hidden");
      };

      const EDITOR_INDENT = "    ";

      const insertIndent = () => {
        const start = editor.selectionStart;
        const end = editor.selectionEnd;
        editor.setRangeText(EDITOR_INDENT, start, end, "end");
        editor.dispatchEvent(new Event("input", { bubbles: true }));
      };

      const isWritableMode = () => !editorPane.classList.contains("hidden");

      const selectAllEditorText = () => {
        editor.focus();
        editor.selectionStart = 0;
        editor.selectionEnd = editor.value.length;
        editor.setSelectionRange(0, editor.value.length);
      };

      const updateEditorLineNumbers = () => {
        const totalLines = Math.max(1, editor.value.split("\n").length);
        editorLineNumbers.innerHTML = Array.from(
          { length: totalLines },
          (_, index) => `<span class="editor-line-number">${index + 1}</span>`
        ).join("");
      };

      const syncEditorScroll = () => {
        editorLineNumbers.scrollTop = editor.scrollTop;
      };

      const moveCaretToLineBoundary = (boundary) => {
        const cursor = editor.selectionStart;
        const value = editor.value;
        const lineStart = value.lastIndexOf("\n", Math.max(0, cursor - 1)) + 1;
        const nextBreak = value.indexOf("\n", cursor);
        const lineEnd = nextBreak === -1 ? value.length : nextBreak;
        const target = boundary === "start" ? lineStart : lineEnd;
        editor.focus();
        editor.setSelectionRange(target, target);
      };

      const lineRangeForSelection = () => {
        const value = editor.value;
        const start = editor.selectionStart;
        const end = editor.selectionEnd;
        const effectiveEnd = end > start && value[end - 1] === "\n" ? end - 1 : end;
        const blockStart = value.lastIndexOf("\n", Math.max(0, start - 1)) + 1;
        let blockEnd = effectiveEnd;

        while (blockEnd < value.length && value[blockEnd] !== "\n") {
          blockEnd += 1;
        }

        return { start, end, blockStart, blockEnd };
      };

      const indentSelectedLines = () => {
        const { start, end, blockStart, blockEnd } = lineRangeForSelection();
        const value = editor.value;
        const block = value.slice(blockStart, blockEnd);

        if (start === end && !block.includes("\n")) {
          insertIndent();
          return;
        }

        const lines = block.split("\n");
        const indented = lines.map((line) => `${EDITOR_INDENT}${line}`).join("\n");
        editor.setRangeText(indented, blockStart, blockEnd, "preserve");

        const nextStart = start + EDITOR_INDENT.length;
        const nextEnd = end + EDITOR_INDENT.length * lines.length;
        editor.setSelectionRange(nextStart, nextEnd);
        editor.dispatchEvent(new Event("input", { bubbles: true }));
      };

      const outdentSelectedLines = () => {
        const { start, end, blockStart, blockEnd } = lineRangeForSelection();
        const value = editor.value;
        const block = value.slice(blockStart, blockEnd);
        const lines = block.split("\n");
        const removedPerLine = lines.map((line) => {
          const match = line.match(/^ {1,4}/);
          return match ? match[0].length : 0;
        });

        if (removedPerLine.every((count) => count === 0)) {
          return;
        }

        const outdented = lines
          .map((line, index) => line.slice(removedPerLine[index]))
          .join("\n");

        editor.setRangeText(outdented, blockStart, blockEnd, "preserve");

        const firstLineRemoved = removedPerLine[0];
        const removedBeforeSelectionEnd = removedPerLine.reduce(
          (total, count) => total + count,
          0
        );
        const nextStart = Math.max(blockStart, start - firstLineRemoved);
        const nextEnd = Math.max(nextStart, end - removedBeforeSelectionEnd);
        editor.setSelectionRange(nextStart, nextEnd);
        editor.dispatchEvent(new Event("input", { bubbles: true }));
      };

      const runEditorCommand = (command) => {
        editor.focus();
        document.execCommand(command);
      };

      const showPaneForMode = (mode) => {
        const hasDocument = mode === "readonly" || mode === "writable";
        emptyState.classList.toggle("hidden", hasDocument);
        previewPane.classList.toggle("hidden", mode !== "readonly");
        editorPane.classList.toggle("hidden", mode !== "writable");
        if (mode !== "readonly") setOutlineOpen(false);
      };

      const updateTabNavigation = () => {
        const overflowed = tabsBar.scrollWidth > tabsBar.clientWidth + 1;
        tabNavigation.classList.toggle("hidden", !overflowed);
        previousTabs.disabled = !overflowed || tabsBar.scrollLeft <= 1;
        nextTabs.disabled =
          !overflowed ||
          tabsBar.scrollLeft + tabsBar.clientWidth >= tabsBar.scrollWidth - 1;
      };

      const revealActiveTab = () => {
        const activeTab = tabsBar.querySelector(".document-tab.active");
        activeTab?.scrollIntoView({ behavior: "smooth", block: "nearest", inline: "nearest" });
      };

      const renderTabs = (tabs) => {
        tabsShell.classList.toggle("hidden", !tabs.length);
        if (!tabs.length) {
          tabsBar.innerHTML = "";
          requestAnimationFrame(updateTabNavigation);
          return;
        }

        tabsBar.innerHTML = tabs
          .map((tab) => {
            const activeClass = tab.active ? " active" : "";
            const dirty = tab.dirty ? `<span class="document-tab__dirty" aria-hidden="true"></span>` : "";
            return `
              <div class="document-tab${activeClass}" role="tab" tabindex="0" aria-selected="${tab.active}" data-document-id="${tab.document_id}" title="${escapeHtml(tab.title)}">
                <span class="document-tab__name">${escapeHtml(tab.file_name)}</span>
                ${dirty}
                <button class="document-tab__close" type="button" data-close-document="${tab.document_id}" data-file-name="${escapeHtml(tab.file_name)}" aria-label="${escapeHtml(appStrings.closeDocument)} ${escapeHtml(tab.file_name)}">&times;</button>
              </div>
            `;
          })
          .join("");
        requestAnimationFrame(() => {
          updateTabNavigation();
          revealActiveTab();
        });
      };

      const setOutlineOpen = (open) => {
        outlineOpen = open && currentDocumentMode === "readonly";
        outlinePanel.classList.toggle("hidden", !outlineOpen);
        workspace.classList.toggle("outline-open", outlineOpen);
      };

      const refreshOutline = () => {
        outlineList.innerHTML = "";
        if (currentDocumentMode !== "readonly") return;
        const headings = Array.from(content.querySelectorAll("h1, h2, h3, h4, h5, h6"));
        if (!headings.length) {
          const empty = document.createElement("p");
          empty.className = "outline-empty";
          empty.textContent = appStrings.noHeadings;
          outlineList.appendChild(empty);
          return;
        }

        for (const heading of headings) {
          const item = document.createElement("button");
          item.type = "button";
          item.className = "outline-item";
          item.dataset.level = heading.tagName.slice(1);
          item.textContent = heading.textContent?.trim() || appStrings.untitledHeading;
          item.addEventListener("click", () => {
            heading.scrollIntoView({ behavior: "smooth", block: "start" });
          });
          outlineList.appendChild(item);
        }
      };

      const resetWorkspaceChrome = (statusMessage) => {
        document.body.classList.add("workspace-empty");
        document.title = "MarkHola";
        documentBase.setAttribute("href", "");
        showPaneForMode(null);
        window.showStatus({ message: statusMessage || appStrings.ready, level: "info" });
      };

      const applyWorkspaceChrome = (payload) => {
        renderTabs(payload.tabs || []);
        const active = payload.active_document;

        if (!active) {
          resetWorkspaceChrome(payload.status_message);
          return;
        }

        document.body.classList.remove("workspace-empty");
        document.title = `${active.file_name}${active.dirty ? " *" : ""} - MarkHola`;
        documentBase.setAttribute("href", active.base_url);
        showPaneForMode(active.mode);
        window.showStatus({ message: payload.status_message, level: active.dirty ? "warning" : "info" });
      };

      const escapeHtml = (value) =>
        value
          .replaceAll("&", "&amp;")
          .replaceAll("<", "&lt;")
          .replaceAll(">", "&gt;")
          .replaceAll('"', "&quot;")
          .replaceAll("'", "&#39;");

      const applyAppTheme = (cssText) => {
        if (!appThemeStyle) return;
        appThemeStyle.textContent = cssText;
      };

      const ensureMermaidInitialized = () => {
        if (mermaidInitialized || !window.mermaid) return;

        window.mermaid.initialize({
          startOnLoad: false,
          securityLevel: "strict",
          theme: "default"
        });
        mermaidInitialized = true;
      };

      const removeMermaidRenderArtifact = (renderId) => {
        document.getElementById(`d${renderId}`)?.remove();
      };

      const renderMermaidDiagrams = async () => {
        ensureMermaidInitialized();
        if (!window.mermaid) return;

        const blocks = document.querySelectorAll(".mermaid-block");
        for (const [index, block] of blocks.entries()) {
          const statusNode = block.querySelector(".mermaid-block__status");
          const sourceNode = block.querySelector(".mermaid-block__source");
          const diagramNode = block.querySelector(".mermaid-block__diagram");
          const source = sourceNode?.textContent || "";

          if (!diagramNode) continue;

          diagramNode.innerHTML = "";
          if (statusNode) {
            statusNode.textContent = appStrings.renderingDiagram;
            statusNode.classList.remove("hidden");
          }

          const renderId = `mermaid-diagram-${index}-${Date.now()}`;
          try {
            const { svg } = await window.mermaid.render(renderId, source);
            diagramNode.innerHTML = svg;
            statusNode?.classList.add("hidden");
          } catch (error) {
            const message =
              error && typeof error === "object" && "message" in error
                ? String(error.message)
                : String(error || "Unknown Mermaid error");
            if (statusNode) {
              statusNode.textContent = appStrings.diagramFailed;
              statusNode.classList.remove("hidden");
            }
            diagramNode.innerHTML =
              `<pre class="mermaid-block__error">${escapeHtml(message)}\n\n${escapeHtml(source)}</pre>`;
          } finally {
            removeMermaidRenderArtifact(renderId);
          }
        }
      };

      const ensureMathJaxReady = () => {
        if (!window.MathJax || !window.MathJax.startup) return null;
        if (!mathJaxReadyPromise) {
          mathJaxReadyPromise = window.MathJax.startup.promise;
        }
        return mathJaxReadyPromise;
      };

      const extractRenderedMathNode = (rendered) =>
        rendered.querySelector("mjx-container") || rendered.firstElementChild || rendered;

      const renderMathSource = async (node, source, display) => {
        const ready = ensureMathJaxReady();
        if (!ready) return false;

        await ready;
        const rendered = await window.MathJax.tex2svgPromise(source, { display });
        const mathNode = extractRenderedMathNode(rendered);
        node.replaceChildren(mathNode.cloneNode(true));
        return true;
      };

      const renderMathExpressions = async () => {
        if (!window.MathJax) return;

        const mathNodes = content.querySelectorAll(".math.math-inline, .math.math-display");
        for (const node of mathNodes) {
          const source = node.textContent || "";
          const display = node.classList.contains("math-display");

          try {
            await renderMathSource(node, source, display);
          } catch (error) {
            const message =
              error && typeof error === "object" && "message" in error
                ? String(error.message)
                : String(error || "Unknown math error");
            node.innerHTML = `<code>${escapeHtml(`Math render failed: ${message}\n\n${source}`)}</code>`;
          }
        }

        const blocks = content.querySelectorAll(".math-block");
        for (const block of blocks) {
          const statusNode = block.querySelector(".math-block__status");
          const sourceNode = block.querySelector(".math-block__source");
          const formulaNode = block.querySelector(".math-block__formula");
          const source = sourceNode?.textContent || "";

          if (!formulaNode) continue;

          formulaNode.innerHTML = "";
          if (statusNode) {
            statusNode.textContent = appStrings.renderingFormula;
            statusNode.classList.remove("hidden");
          }

          try {
            await renderMathSource(formulaNode, source, true);
            statusNode?.classList.add("hidden");
          } catch (error) {
            const message =
              error && typeof error === "object" && "message" in error
                ? String(error.message)
                : String(error || "Unknown math error");
            if (statusNode) {
              statusNode.textContent = appStrings.formulaFailed;
              statusNode.classList.remove("hidden");
            }
            formulaNode.innerHTML =
              `<pre class="math-block__error">${escapeHtml(message)}\n\n${escapeHtml(source)}</pre>`;
          }
        }
      };

      const renderReadonlyEnhancements = async () => {
        await renderMermaidDiagrams();
        await renderMathExpressions();
      };

      const hasActiveDocument = () => currentDocumentId !== null;

      const activeFindQuery = () => findInput.value;

      const updateFindCount = (matchCount, activeIndex) => {
        if (matchCount <= 0) {
          findCount.textContent = appStrings.resultsZero;
          return;
        }
        if (matchCount === 1) {
          findCount.textContent = appStrings.resultsOne;
          return;
        }
        findCount.textContent = `${activeIndex + 1} / ${matchCount} ${appStrings.resultsMany}`;
      };

      const updateFindControls = () => {
        const query = activeFindQuery();
        const matchCount = currentDocumentMode === "writable" ? writableMatches.length : readonlyMatches.length;
        const hasMatches = query.length > 0 && matchCount > 0;
        const writable = currentDocumentMode === "writable";

        findPrevious.disabled = !hasMatches;
        findNext.disabled = !hasMatches;
        replaceOne.disabled = !writable || !hasMatches;
        replaceAll.disabled = !writable || !hasMatches;
      };

      const syncFindPanelMode = () => {
        const writable = currentDocumentMode === "writable";
        replaceGroup.classList.toggle("hidden", !writable);
        replaceOne.classList.toggle("hidden", !writable);
        replaceAll.classList.toggle("hidden", !writable);
        updateFindControls();
      };

      const restoreReadonlyBaseHtml = () => {
        if (readonlyBaseHtml) {
          content.innerHTML = readonlyBaseHtml;
        }
      };

      const resetReadonlyMatches = (restoreBaseHtml = true) => {
        readonlyMatches = [];
        readonlyActiveIndex = -1;
        if (restoreBaseHtml) {
          restoreReadonlyBaseHtml();
        }
      };

      const resetWritableMatches = () => {
        writableMatches = [];
        writableActiveIndex = -1;
      };

      const closeFindPanel = () => {
        findPanelVisible = false;
        findPanel.classList.add("hidden");
        resetReadonlyMatches(true);
        resetWritableMatches();
        updateFindCount(0, -1);
        updateFindControls();
      };

      const openFindPanel = () => {
        if (!hasActiveDocument()) {
          window.showStatus({ message: appStrings.noDocumentError, level: "error" });
          return;
        }

        findPanelVisible = true;
        findPanel.classList.remove("hidden");
        syncFindPanelMode();

        if (currentDocumentMode === "readonly") {
          refreshReadonlyFindResults();
        } else {
          refreshWritableFindResults(null, false);
        }

        requestAnimationFrame(() => {
          findInput.focus();
          findInput.select();
        });
      };

      const collectWritableMatches = (source, query) => {
        if (!query) {
          return [];
        }

        const ranges = [];
        const lowerSource = source.toLowerCase();
        const lowerQuery = query.toLowerCase();
        let cursor = 0;

        while (cursor <= lowerSource.length - lowerQuery.length) {
          const index = lowerSource.indexOf(lowerQuery, cursor);
          if (index === -1) {
            break;
          }

          ranges.push({ start: index, end: index + query.length });
          cursor = index + query.length;
        }

        return ranges;
      };

      const selectWritableMatch = (index, focusEditor = true) => {
        if (index < 0 || index >= writableMatches.length) {
          return;
        }

        writableActiveIndex = index;
        const match = writableMatches[index];
        if (focusEditor) {
          editor.focus();
        }
        editor.setSelectionRange(match.start, match.end);
        updateFindCount(writableMatches.length, writableActiveIndex);
        updateFindControls();
      };

      function refreshWritableFindResults(preferredStart = null, focusEditor = false) {
        resetReadonlyMatches(true);
        const query = activeFindQuery();

        if (!findPanelVisible || currentDocumentMode !== "writable" || !query) {
          resetWritableMatches();
          updateFindCount(0, -1);
          updateFindControls();
          return;
        }

        writableMatches = collectWritableMatches(editor.value, query);
        if (!writableMatches.length) {
          writableActiveIndex = -1;
          updateFindCount(0, -1);
          updateFindControls();
          return;
        }

        const nextIndex =
          typeof preferredStart === "number"
            ? writableMatches.findIndex((match) => match.start >= preferredStart)
            : -1;

        if (nextIndex >= 0) {
          writableActiveIndex = nextIndex;
        } else if (writableActiveIndex < 0 || writableActiveIndex >= writableMatches.length) {
          writableActiveIndex = 0;
        }

        selectWritableMatch(writableActiveIndex, focusEditor);
      }

      const activateReadonlyMatch = (index) => {
        if (index < 0 || index >= readonlyMatches.length) {
          return;
        }

        readonlyMatches.forEach((match) => match.classList.remove("find-match--active"));
        readonlyActiveIndex = index;
        const activeMatch = readonlyMatches[index];
        activeMatch.classList.add("find-match--active");
        activeMatch.scrollIntoView({ block: "nearest", inline: "nearest" });
        updateFindCount(readonlyMatches.length, readonlyActiveIndex);
        updateFindControls();
      };

      const isSearchableReadonlyTextNode = (node) => {
        const parent = node.parentElement;
        if (!parent || !node.textContent || !node.textContent.trim()) {
          return false;
        }

        if (parent.namespaceURI !== "http://www.w3.org/1999/xhtml") {
          return false;
        }

        return !parent.closest(".hidden, .mermaid-block__source, .math-block__source, script, style, textarea");
      };

      function refreshReadonlyFindResults() {
        resetWritableMatches();
        const query = activeFindQuery();

        if (!findPanelVisible || currentDocumentMode !== "readonly" || !query || !readonlyBaseHtml) {
          resetReadonlyMatches(true);
          updateFindCount(0, -1);
          updateFindControls();
          return;
        }

        resetReadonlyMatches(true);

        const lowerQuery = query.toLowerCase();
        const walker = document.createTreeWalker(content, NodeFilter.SHOW_TEXT, {
          acceptNode(node) {
            return isSearchableReadonlyTextNode(node)
              ? NodeFilter.FILTER_ACCEPT
              : NodeFilter.FILTER_REJECT;
          }
        });

        const textNodes = [];
        while (walker.nextNode()) {
          textNodes.push(walker.currentNode);
        }

        for (const node of textNodes) {
          const source = node.textContent || "";
          const lowerSource = source.toLowerCase();
          let searchStart = 0;
          let matchIndex = lowerSource.indexOf(lowerQuery, searchStart);

          if (matchIndex === -1) {
            continue;
          }

          const fragment = document.createDocumentFragment();
          while (matchIndex !== -1) {
            if (matchIndex > searchStart) {
              fragment.append(document.createTextNode(source.slice(searchStart, matchIndex)));
            }

            const mark = document.createElement("mark");
            mark.className = "find-match";
            mark.textContent = source.slice(matchIndex, matchIndex + query.length);
            fragment.append(mark);
            readonlyMatches.push(mark);

            searchStart = matchIndex + query.length;
            matchIndex = lowerSource.indexOf(lowerQuery, searchStart);
          }

          if (searchStart < source.length) {
            fragment.append(document.createTextNode(source.slice(searchStart)));
          }

          node.parentNode.replaceChild(fragment, node);
        }

        if (!readonlyMatches.length) {
          updateFindCount(0, -1);
          updateFindControls();
          return;
        }

        if (readonlyActiveIndex < 0 || readonlyActiveIndex >= readonlyMatches.length) {
          readonlyActiveIndex = 0;
        }
        activateReadonlyMatch(readonlyActiveIndex);
      }

      const stepFindResult = (direction) => {
        if (!findPanelVisible || !activeFindQuery()) {
          return;
        }

        if (currentDocumentMode === "writable") {
          if (!writableMatches.length) {
            updateFindControls();
            return;
          }

          const nextIndex =
            (writableActiveIndex + direction + writableMatches.length) % writableMatches.length;
          selectWritableMatch(nextIndex, true);
          return;
        }

        if (!readonlyMatches.length) {
          updateFindControls();
          return;
        }

        const nextIndex =
          (readonlyActiveIndex + direction + readonlyMatches.length) % readonlyMatches.length;
        activateReadonlyMatch(nextIndex);
      };

      const replaceCurrentWritableMatch = () => {
        if (currentDocumentMode !== "writable" || writableActiveIndex < 0 || writableActiveIndex >= writableMatches.length) {
          updateFindControls();
          return;
        }

        const match = writableMatches[writableActiveIndex];
        const replacement = replaceInput.value;

        editor.focus();
        editor.setSelectionRange(match.start, match.end);
        editor.setRangeText(replacement, match.start, match.end, "end");
        editor.dispatchEvent(new Event("input", { bubbles: true }));
        refreshWritableFindResults(match.start + replacement.length, true);
      };

      const replaceAllWritableMatches = () => {
        const query = activeFindQuery();
        if (currentDocumentMode !== "writable" || !query || !writableMatches.length) {
          updateFindControls();
          return;
        }

        const replacement = replaceInput.value;
        let cursor = 0;
        let nextValue = "";

        for (const match of writableMatches) {
          nextValue += editor.value.slice(cursor, match.start);
          nextValue += replacement;
          cursor = match.end;
        }

        nextValue += editor.value.slice(cursor);

        editor.value = nextValue;
        editor.focus();
        editor.dispatchEvent(new Event("input", { bubbles: true }));
        refreshWritableFindResults(0, true);
      };

      const finalizeReadonlyRender = async (documentId) => {
        rewriteLocalAssetUrls(documentId);
        await renderReadonlyEnhancements();
        if (documentId !== currentDocumentId || currentDocumentMode !== "readonly") {
          return;
        }

        readonlyBaseHtml = content.innerHTML;
        if (findPanelVisible) {
          refreshReadonlyFindResults();
        } else {
          resetReadonlyMatches(false);
        }
      };

      const rewriteLocalAssetUrls = (documentId) => {
        const baseHref = documentBase.getAttribute("href") || "";
        window.ipc.postMessage(JSON.stringify({ kind: "debug-log", message: `rewriteLocalAssetUrls baseHref=${baseHref}` }));
        if (!baseHref) return;

        const images = content.querySelectorAll("img[src]");
        for (const image of images) {
          const src = image.getAttribute("src") || "";
          if (!src) continue;
          if (
            src.startsWith("http://") ||
            src.startsWith("https://") ||
            src.startsWith("data:") ||
            src.startsWith("blob:")
          ) {
            continue;
          }

          try {
            const baseUrl = new URL(baseHref);
            const fileUrl = new URL(src, baseUrl);
            if (fileUrl.protocol === "file:") {
              const basePath = baseUrl.pathname.endsWith("/") ? baseUrl.pathname : `${baseUrl.pathname}/`;
              if (!fileUrl.pathname.startsWith(basePath)) {
                image.setAttribute("src", `markhola-file://asset/${documentId}/`);
                continue;
              }
              const relativePath = fileUrl.pathname.slice(basePath.length);
              const rewritten = `markhola-file://asset/${documentId}/${relativePath}`;
              image.setAttribute("src", rewritten);
              image.setAttribute("data-original-src", src);
              window.ipc.postMessage(
                JSON.stringify({
                  kind: "debug-log",
                  message: `rewriteLocalAssetUrls src=${src} resolved=${resolved} rewritten=${rewritten}`
                })
              );
            }
          } catch {
            // ignore
          }
        }
      };

      const resolveLocalMarkdownLink = (href) => {
        const baseHref = documentBase.getAttribute("href") || "";
        if (!href || !baseHref) return null;

        try {
          const fileUrl = new URL(href, baseHref);
          const pathname = decodeURIComponent(fileUrl.pathname || "").toLowerCase();
          if (fileUrl.protocol !== "file:") return null;
          if (!pathname.endsWith(".md") && !pathname.endsWith(".markdown")) return null;
          return fileUrl.href;
        } catch {
          return null;
        }
      };

      aboutClose.addEventListener("click", hideAbout);
      aboutOverlay.addEventListener("click", (event) => {
        if (event.target === aboutOverlay) hideAbout();
      });

      aboutCopy.addEventListener("click", async () => {
        const url = aboutGithub.getAttribute("href") || "";
        if (!url) return;

        try {
          await navigator.clipboard.writeText(url);
          aboutCopy.textContent = appStrings.copied;
          setTimeout(() => {
            aboutCopy.textContent = appStrings.copy;
          }, 1200);
        } catch {
          aboutCopy.textContent = appStrings.failed;
          setTimeout(() => {
            aboutCopy.textContent = appStrings.copy;
          }, 1200);
        }
      });

      editor.addEventListener("input", () => {
        updateEditorLineNumbers();
        window.ipc.postMessage(JSON.stringify({ kind: "editor-changed", markdown: editor.value }));
        if (findPanelVisible && currentDocumentMode === "writable") {
          refreshWritableFindResults(editor.selectionStart, true);
        }
      });

      editor.addEventListener("scroll", syncEditorScroll);
      tabsBar.addEventListener("scroll", updateTabNavigation);
      window.addEventListener("resize", updateTabNavigation);
      previousTabs.addEventListener("click", () => {
        tabsBar.scrollBy({ left: -360, behavior: "smooth" });
      });
      nextTabs.addEventListener("click", () => {
        tabsBar.scrollBy({ left: 360, behavior: "smooth" });
      });
      newDocumentTab.addEventListener("click", () => {
        window.ipc.postMessage(JSON.stringify({ kind: "new-document" }));
      });
      outlineClose.addEventListener("click", () => {
        window.ipc.postMessage(JSON.stringify({ kind: "toggle-outline" }));
      });

      findInput.addEventListener("input", () => {
        if (currentDocumentMode === "writable") {
          refreshWritableFindResults(null, false);
        } else {
          refreshReadonlyFindResults();
        }
      });

      findPrevious.addEventListener("click", () => stepFindResult(-1));
      findNext.addEventListener("click", () => stepFindResult(1));
      findClose.addEventListener("click", closeFindPanel);
      replaceOne.addEventListener("click", replaceCurrentWritableMatch);
      replaceAll.addEventListener("click", replaceAllWritableMatches);

      document.addEventListener("keydown", (event) => {
        if (event.key === "Escape" && !aboutOverlay.classList.contains("hidden")) {
          hideAbout();
          return;
        }

        if (findPanelVisible && event.key === "Escape") {
          event.preventDefault();
          closeFindPanel();
          return;
        }

        const focusedTab = event.target.closest?.("[data-document-id]");
        if (focusedTab && (event.key === "Enter" || event.key === " ")) {
          event.preventDefault();
          const documentId = Number(focusedTab.getAttribute("data-document-id"));
          if (Number.isFinite(documentId)) {
            window.ipc.postMessage(JSON.stringify({ kind: "activate-document", documentId }));
          }
          return;
        }

        if (event.target === findInput && event.key === "Enter") {
          event.preventDefault();
          stepFindResult(event.shiftKey ? -1 : 1);
          return;
        }

        if (event.target === editor && event.key === "Tab" && !event.metaKey && !event.ctrlKey) {
          event.preventDefault();
          if (event.shiftKey) {
            outdentSelectedLines();
          } else {
            indentSelectedLines();
          }
          return;
        }

        if (event.target === editor && event.ctrlKey && !event.metaKey && !event.altKey) {
          if (event.key.toLowerCase() === "a") {
            event.preventDefault();
            moveCaretToLineBoundary("start");
            return;
          }

          if (event.key.toLowerCase() === "e") {
            event.preventDefault();
            moveCaretToLineBoundary("end");
            return;
          }
        }

        if (!event.metaKey || event.ctrlKey || event.altKey) {
          return;
        }

        if (event.key.toLowerCase() === "z" && isWritableMode()) {
          if (document.activeElement !== editor) {
            event.preventDefault();
            runEditorCommand("undo");
          }
        } else if (event.key.toLowerCase() === "r" && isWritableMode()) {
          event.preventDefault();
          runEditorCommand("redo");
        } else if (event.key.toLowerCase() === "f") {
          event.preventDefault();
          window.ipc.postMessage(JSON.stringify({ kind: "request-open-find" }));
        } else if (event.key.toLowerCase() === "s") {
          event.preventDefault();
          window.ipc.postMessage(JSON.stringify({ kind: "request-save" }));
        } else if (event.key.toLowerCase() === "p") {
          event.preventDefault();
          window.ipc.postMessage(JSON.stringify({ kind: "request-print" }));
        } else if (event.key.toLowerCase() === "w") {
          event.preventDefault();
          window.ipc.postMessage(JSON.stringify({ kind: "close-current-document" }));
        } else if (event.key.toLowerCase() === "a" && isWritableMode()) {
          event.preventDefault();
          selectAllEditorText();
        } else if (event.key === "/") {
          event.preventDefault();
          window.ipc.postMessage(JSON.stringify({ kind: "toggle-mode" }));
        }
      });

      document.addEventListener("click", (event) => {
        const statusAction = event.target.closest("[data-open-path]");
        if (statusAction) {
          event.preventDefault();
          const path = statusAction.getAttribute("data-open-path") || "";
          if (path) {
            window.ipc.postMessage(JSON.stringify({ kind: "open-external", href: path }));
          }
          return;
        }

        const closeButton = event.target.closest("[data-close-document]");
        if (closeButton) {
          event.preventDefault();
          event.stopPropagation();
          window.ipc.postMessage(
            JSON.stringify({
              kind: "close-document",
              documentId: Number(closeButton.getAttribute("data-close-document"))
            })
          );
          return;
        }

        const tab = event.target.closest("[data-document-id]");
        if (tab) {
          const documentId = Number(tab.getAttribute("data-document-id"));
          if (Number.isFinite(documentId)) {
            window.ipc.postMessage(JSON.stringify({ kind: "activate-document", documentId }));
          }
          return;
        }

        const link = event.target.closest("a[href]");
        if (!link) return;

        const href = link.getAttribute("href") || "";
        if (href.startsWith("#")) {
          const targetId = href.slice(1);
          if (targetId) {
            const target = document.getElementById(targetId);
            if (target) {
              event.preventDefault();
              target.scrollIntoView({ block: "start", inline: "nearest" });
              history.replaceState(null, "", href);
              return;
            }
          }
        }
        if (href.startsWith("http://") || href.startsWith("https://")) {
          event.preventDefault();
          window.ipc.postMessage(JSON.stringify({ kind: "open-external", href }));
          return;
        }

        const localMarkdownHref = resolveLocalMarkdownLink(href);
        if (localMarkdownHref) {
          event.preventDefault();
          window.ipc.postMessage(
            JSON.stringify({ kind: "open-markdown-link", href: localMarkdownHref })
          );
        }
      });

      document.addEventListener(
        "error",
        (event) => {
          const target = event.target;
          if (!(target instanceof HTMLImageElement)) return;

          window.ipc.postMessage(
            JSON.stringify({
              kind: "debug-log",
              message: `image-error src=${target.getAttribute("src") || ""}`
            })
          );

          const fallback = document.createElement("p");
          fallback.className = "image-error";
          const originalSrc = target.getAttribute("data-original-src");
          const visibleSrc = originalSrc || target.getAttribute("src") || "unknown source";
          fallback.textContent = `${appStrings.imageFailed} ${visibleSrc}`;
          target.replaceWith(fallback);
        },
        true
      );

      window.showStatus = (payload) => {
        const actionPath = payload.action_path || "";
        const actionLabel = payload.action_label || "";
        if (actionPath && actionLabel) {
          status.innerHTML = `${escapeHtml(payload.message)} <a href=\"#\" class=\"status__action\" data-open-path=\"${escapeHtml(actionPath)}\">${escapeHtml(actionLabel)}</a>`;
        } else {
          status.textContent = payload.message;
        }
        status.dataset.level = payload.level || "info";
      };

      const applyWorkspacePayload = (payload, forceRefresh) => {
        applyWorkspaceChrome(payload);
        const active = payload.active_document;
        const nextDocumentId = active ? active.document_id : null;
        const documentChanged = nextDocumentId !== currentDocumentId;
        const modeChanged = !!active && active.mode !== currentDocumentMode;

        if (!active) {
          currentDocumentId = null;
          currentDocumentMode = null;
          readonlyBaseHtml = "";
          content.innerHTML = "";
          editor.value = "";
          updateEditorLineNumbers();
          syncEditorScroll();
          closeFindPanel();
          setOutlineOpen(false);
          outlineList.innerHTML = "";
          return;
        }

        if (documentChanged || modeChanged) {
          resetReadonlyMatches(true);
          resetWritableMatches();
        }

        if (forceRefresh || documentChanged || active.mode === "readonly") {
          content.innerHTML = active.html;
          readonlyBaseHtml = active.html;
        }

        if (forceRefresh || documentChanged) {
          editor.value = active.markdown;
          updateEditorLineNumbers();
          syncEditorScroll();
        }

        currentDocumentId = nextDocumentId;
        currentDocumentMode = active.mode;
        syncFindPanelMode();

        if (forceRefresh || documentChanged || active.mode === "readonly") {
          refreshOutline();
          void finalizeReadonlyRender(nextDocumentId);
        } else if (findPanelVisible && active.mode === "writable") {
          refreshWritableFindResults(editor.selectionStart, true);
        }
      };

      window.renderWorkspace = (payload) => {
        applyWorkspacePayload(payload, true);
      };

      window.updateWorkspaceState = (payload) => {
        applyWorkspacePayload(payload, false);
      };

      updateEditorLineNumbers();

      window.showAbout = (payload) => {
        aboutVersion.textContent = payload.version;
        aboutAuthor.textContent = payload.author;
        aboutBuild.textContent = `${payload.buildPlatform} / ${payload.buildTarget}`;
        aboutGithub.textContent = payload.githubUrl;
        aboutGithub.setAttribute("href", payload.githubUrl);
        aboutCopy.textContent = appStrings.copy;
        aboutOverlay.classList.remove("hidden");
      };

      window.openFindPanel = openFindPanel;
      window.applyAppTheme = applyAppTheme;
      window.applyAppLanguage = applyAppLanguage;
      window.applyDocumentSize = applyDocumentSize;
      window.setOutlinePanelOpen = (open) => {
        if (currentDocumentMode !== "readonly") {
          setOutlineOpen(false);
          return;
        }
        refreshOutline();
        setOutlineOpen(Boolean(open));
      };

      applyDocumentSize(__DOCUMENT_SIZE__);
      applyAppLanguage(appStrings);
      window.ipc.postMessage(JSON.stringify({ kind: "shell-ready" }));
    </script>
  </body>
</html>
