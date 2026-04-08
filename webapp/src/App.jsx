import { useEffect, useState } from 'react';

const copyBlocks = {
  install: 'curl -fsSL https://raw.githubusercontent.com/SyedSibtainRazvi/context0/main/install.sh | sh',
  initRules: 'context0 init-rules',
  mcpQuickConfig: `# Claude Code (run once in terminal)
claude mcp add context0 context0 mcp-server

# Cursor / Codex — use full path (editors don't inherit PATH)
# First run: which context0
# Then add to ~/.cursor/mcp.json or ~/.codex/config.json:
{
  "mcpServers": {
    "context0": {
      "command": "/Users/your-username/.local/bin/context0",
      "args": ["mcp-server"]
    }
  }
}`,
  save: 'context0 save --done "wired auth middleware" --next "fix integration tests"',
  claudeConfig: 'claude mcp add context0 context0 mcp-server',
  cursorConfig: `{
  "mcpServers": {
    "context0": {
      "command": "/Users/your-username/.local/bin/context0",
      "args": ["mcp-server"]
    }
  }
}`,
  codexConfig: `{
  "mcpServers": {
    "context0": {
      "command": "/Users/your-username/.local/bin/context0",
      "args": ["mcp-server"]
    }
  }
}`,
};

function CopyButton({ text }) {
  const [label, setLabel] = useState('Copy');

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(text);
      setLabel('Copied');
    } catch {
      setLabel('Failed');
    }

    window.setTimeout(() => {
      setLabel('Copy');
    }, 1100);
  }

  return (
    <button className="copy-btn" onClick={handleCopy} type="button">
      {label}
    </button>
  );
}

function CodeWrap({ text, compact = false }) {
  return (
    <div className={`code-wrap${compact ? ' compact' : ''}`}>
      <pre>
        <code>{text}</code>
      </pre>
      <CopyButton text={text} />
    </div>
  );
}

export default function App() {
  const [menuOpen, setMenuOpen] = useState(false);
  const [activeTab, setActiveTab] = useState('claude');

  useEffect(() => {
    const revealEls = Array.from(document.querySelectorAll('.reveal'));
    const sectionIds = ['what', 'quickstart', 'mcp', 'manual'];
    const navLinks = Array.from(document.querySelectorAll('.topnav a'));

    const revealObserver = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (!entry.isIntersecting) return;
          entry.target.classList.add('visible');
          revealObserver.unobserve(entry.target);
        });
      },
      { threshold: 0.15 }
    );

    revealEls.forEach((el, idx) => {
      el.style.animationDelay = `${Math.min(idx * 70, 300)}ms`;
      revealObserver.observe(el);
    });

    const sectionObserver = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (!entry.isIntersecting) return;
          const activeId = entry.target.id;
          navLinks.forEach((link) => {
            const href = link.getAttribute('href');
            link.classList.toggle('active', href === `#${activeId}`);
          });
        });
      },
      {
        rootMargin: '-30% 0px -55% 0px',
        threshold: 0.01,
      }
    );

    sectionIds.forEach((id) => {
      const section = document.getElementById(id);
      if (section) sectionObserver.observe(section);
    });

    return () => {
      revealObserver.disconnect();
      sectionObserver.disconnect();
    };
  }, []);

  function closeMenu() {
    setMenuOpen(false);
  }

  return (
    <>
      <div className="bg-orb bg-orb-a"></div>
      <div className="bg-orb bg-orb-b"></div>

      <header className="topbar">
        <a className="brand" href="#top">
          <span className="brand-mark">C</span>
          <span>context0</span>
        </a>
        <button
          aria-expanded={menuOpen}
          aria-label="Open navigation"
          className="menu-toggle"
          onClick={() => setMenuOpen((open) => !open)}
          type="button"
        >
          Menu
        </button>
        <nav className={`topnav${menuOpen ? ' open' : ''}`} id="topnav">
          {[
            ['#what', 'What is it'],
            ['#quickstart', 'Quickstart'],
            ['#mcp', 'MCP setup'],
            ['#manual', 'Manual fallback'],
          ].map(([href, label]) => (
            <a href={href} key={href} onClick={closeMenu}>
              {label}
            </a>
          ))}
        </nav>
      </header>

      <main id="top">
        <section className="hero reveal">
          <p className="kicker">git-scoped session state</p>
          <h1>Your AI context follows your git branch.</h1>
          <p className="lead">
            Save where you left off. Resume in any AI tool — Claude Code, Cursor, Codex — on the
            same repo and branch, automatically.
          </p>
          <p className="hint">
            After one-time MCP setup, just talk to your agent normally. You do not need to
            manually run <code>context0 save</code> or <code>context0 resume</code>.
          </p>
          <div className="hero-tags">
            {['Open Source', 'Free', 'No Signup', 'No Cloud', 'Local-only', 'MIT License'].map(
              (tag) => (
                <span className="tag" key={tag}>
                  {tag}
                </span>
              )
            )}
          </div>
          <div className="hero-actions">
            <a className="btn btn-primary" href="#quickstart">
              Start in 2 minutes
            </a>
            <a className="btn btn-ghost" href="#mcp">
              MCP setup
            </a>
            <a
              className="btn btn-ghost"
              href="https://github.com/SyedSibtainRazvi/context0"
              rel="noopener"
              target="_blank"
            >
              Star on GitHub
            </a>
          </div>
        </section>

        <section className="panel reveal" id="what">
          <h2>What is context0?</h2>
          <p>
            AI coding sessions have no memory between tools. <code>context0</code> stores compact
            checkpoints in local SQLite, scoped to your git repo and branch. With MCP configured,
            the agent saves and resumes automatically — no commands to run, nothing to learn.
          </p>
          <div className="grid three">
            <article>
              <h3>Automatic</h3>
              <p>
                Talk to Claude Code, Cursor, or Codex normally. If you say you&apos;re switching
                tools or ending the session, the agent handles save/resume through MCP.
              </p>
            </article>
            <article>
              <h3>Local</h3>
              <p>
                No cloud, no account, no signup. Data stays on your machine at{' '}
                <code>~/.context0/context0.db</code>.
              </p>
            </article>
            <article>
              <h3>Scoped</h3>
              <p>
                Context key is <code>repo_root + branch</code> — each branch stays completely
                isolated.
              </p>
            </article>
          </div>
        </section>

        <section className="panel reveal" id="quickstart">
          <h2>Quickstart</h2>
          <h3>With MCP — recommended</h3>
          <p>
            Set it up once. After that, talk to the agent normally — no need to manually run{' '}
            <code>context0 save</code> or <code>context0 resume</code>. The agent handles
            everything through MCP.
          </p>
          <ol className="steps">
            <li>
              <strong>Install</strong>
              <CodeWrap text={copyBlocks.install} />
              <p className="hint">
                Supports macOS (Intel and Apple Silicon) and Linux (<code>x86_64</code> and{' '}
                <code>arm64</code>). Windows: download the <code>.zip</code> from{' '}
                <a href="https://github.com/SyedSibtainRazvi/context0/releases">Releases</a> and
                add the binary to your PATH.
              </p>
            </li>
            <li>
              <strong>Install rule files</strong> (once per project)
              <CodeWrap text={copyBlocks.initRules} />
              <p className="hint">
                Writes <code>CLAUDE.md</code>, <code>.cursor/rules/context0.mdc</code>, and{' '}
                <code>AGENTS.md</code> into the current project.
              </p>
            </li>
            <li>
              <strong>Add MCP config</strong> to your tool
              <CodeWrap text={copyBlocks.mcpQuickConfig} />
              <p className="hint">
                See the <a href="#mcp">MCP setup</a> section below for per-tool details.
              </p>
            </li>
          </ol>
        </section>

        <section className="panel reveal" id="mcp">
          <h2>MCP setup</h2>
          <p>
            Add <code>context0 mcp-server</code> to your editor&apos;s MCP config. The agent then
            calls <code>get_context</code>, <code>save_context</code>, and{' '}
            <code>list_context</code> automatically.
          </p>

          <div aria-label="MCP Config Tabs" className="tab-controls" role="tablist">
            {[
              ['claude', 'Claude Code'],
              ['cursor', 'Cursor'],
              ['codex', 'Codex'],
            ].map(([name, label]) => (
              <button
                aria-selected={activeTab === name}
                className={`tab-btn${activeTab === name ? ' active' : ''}`}
                key={name}
                onClick={() => setActiveTab(name)}
                role="tab"
                type="button"
              >
                {label}
              </button>
            ))}
          </div>

          <div className={`tab-panel${activeTab === 'claude' ? ' active' : ''}`}>
            <CodeWrap text={copyBlocks.claudeConfig} />
            <p className="hint">
              Run once in any terminal. Claude Code resolves the binary from your PATH
              automatically.
            </p>
          </div>

          <div className={`tab-panel${activeTab === 'cursor' ? ' active' : ''}`}>
            <CodeWrap text={copyBlocks.cursorConfig} />
            <p className="hint">
              File: <code>~/.cursor/mcp.json</code> (global) or <code>.cursor/mcp.json</code>{' '}
              (project). Cursor doesn&apos;t inherit your shell PATH — use the full binary path.
              Run <code>which context0</code> to find yours, then restart Cursor.
            </p>
          </div>

          <div className={`tab-panel${activeTab === 'codex' ? ' active' : ''}`}>
            <CodeWrap text={copyBlocks.codexConfig} />
            <p className="hint">
              File: <code>~/.codex/config.json</code> &nbsp;&middot;&nbsp; Use the full path from{' '}
              <code>which context0</code>.
            </p>
          </div>
        </section>

        <section className="panel reveal" id="manual">
          <h2>Manual Fallback</h2>
          <p>
            Prefer not to use MCP? You can still save and resume manually from a terminal, but
            that is the fallback path, not the recommended one.
          </p>
          <CodeWrap text={copyBlocks.save} />
          <p className="hint">
            Resume with <code>context0 resume</code> or <code>context0 resume --json</code>.
            Detailed CLI docs live in the README.
          </p>
        </section>
      </main>

      <footer className="site-footer reveal">
        <p>Open source &middot; MIT License &middot; No account required</p>
      </footer>
    </>
  );
}
