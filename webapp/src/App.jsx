import { useEffect, useState } from 'react';

const copyBlocks = {
  install: 'curl -fsSL https://raw.githubusercontent.com/SyedSibtainRazvi/context0/main/install.sh | sh',
  initRules: 'context0 init-rules',
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
  const [copied, setCopied] = useState(false);

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
    } catch {
      // ignore
    }

    window.setTimeout(() => {
      setCopied(false);
    }, 1100);
  }

  return (
    <button
      className="copy-btn"
      data-copied={copied || undefined}
      onClick={handleCopy}
      type="button"
    >
      {copied ? 'Copied' : 'Copy'}
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
          <p className="kicker">context handoff for AI coding agents</p>
          <h1>Your AI agent picks up exactly where it left off.</h1>
          <p className="lead">
            Every new AI session starts cold. context0 fixes that.
          </p>
          <p className="tagline-pun">
            Like <code>git stash</code>, but for your agent&apos;s working memory.
          </p>
          <div className="hero-tags">
            {['Open Source', 'Free', 'No Signup', 'No Cloud', 'MIT License'].map(
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
          <h2>How it works</h2>
          <p>
            Run <code>context0 init-rules</code> once per project and configure MCP for your
            editor. That&apos;s the whole setup. After that:
          </p>
          <ul className="how-list">
            <li>
              When you start a session, the agent is instructed to call <code>get_context</code>{' '}
              and read the latest checkpoint for the current branch.
            </li>
            <li>
              When you say &ldquo;I&rsquo;m switching to Cursor&rdquo; or &ldquo;save my
              session&rdquo;, the agent calls <code>save_context</code> with a structured summary:
              what&apos;s done, what&apos;s next, blockers, test status, and key files.
            </li>
            <li>
              Open the same branch in another tool. It loads the same checkpoint and resumes from
              there.
            </li>
          </ul>
          <video
            autoPlay
            className="demo-screenshot"
            loop
            muted
            playsInline
            poster="https://github.com/user-attachments/assets/cd17149f-3199-4b66-9f06-fc7f142a1138"
            preload="none"
          >
            <source src="/context0.mp4" type="video/mp4" />
          </video>
          <div className="grid three">
            <article>
              <h3>Not long-term memory</h3>
              <p>
                context0 is not a knowledge base or conversation log. It is a short-lived handoff
                note, the minimum state needed to resume a task cleanly in a fresh agent session.
              </p>
            </article>
            <article>
              <h3>Not static rules</h3>
              <p>
                It is not <code>CLAUDE.md</code> or <code>AGENTS.md</code>. Those are permanent
                project instructions. context0 is ephemeral, scoped to the current branch. Each
                save appends a new checkpoint; resume always reads the latest one.
              </p>
            </article>
            <article>
              <h3>Local only, no cloud</h3>
              <p>
                Checkpoints are stored in SQLite at <code>~/.context0/context0.db</code>.{' '}
                <code>feature/auth</code> and <code>main</code> stay completely separate. Nothing
                leaves your machine.
              </p>
            </article>
          </div>
        </section>

        <section className="panel reveal" id="quickstart">
          <h2>Quickstart</h2>
          <p>
            Three steps, done once. After that, the agent handles everything automatically through
            MCP. No manual <code>context0</code> commands needed.
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
                Writes agent instruction files (<code>CLAUDE.md</code>,{' '}
                <code>.cursor/rules/context0.mdc</code>, <code>AGENTS.md</code>) that tell the
                agent to automatically load your checkpoint on session start and save it when you
                switch tools. No manual prompting needed.
              </p>
            </li>
            <li>
              <strong>Add MCP config</strong> to your tool
              <p className="hint" style={{ marginTop: '0.4rem' }}>
                See the <a href="#mcp">MCP setup</a> section below. Pick your tool, Claude Code,
                Cursor, or Codex, and copy the one-liner or JSON snippet.
              </p>
            </li>
          </ol>
        </section>

        <section className="panel reveal" id="mcp">
          <h2>MCP setup</h2>
          <p>
            Add <code>context0 mcp-server</code> to your editor&apos;s MCP config. After that, the
            agent can save and load progress automatically.
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
              (project). Cursor does not inherit your shell PATH, so use the full binary path.
              Run <code>which context0</code> to find yours, then restart Cursor.
            </p>
          </div>

          <div className={`tab-panel${activeTab === 'codex' ? ' active' : ''}`}>
            <CodeWrap text={copyBlocks.codexConfig} />
            <p className="hint">
              File: <code>~/.codex/config.json</code>. Use the full path from{' '}
              <code>which context0</code>.
            </p>
          </div>
        </section>

        <section className="panel reveal" id="manual">
          <h2>Manual Fallback</h2>
          <p>
            Prefer not to use MCP? You can still save and resume from a terminal. That is the
            fallback path, not the main workflow.
          </p>
          <CodeWrap text={copyBlocks.save} />
          <p className="hint">
            Resume with <code>context0 resume</code> or <code>context0 resume --json</code>.
            Detailed CLI docs live in the README.
          </p>
        </section>
      </main>

      <footer className="site-footer reveal">
        <p>
          Open source. MIT License. No account required.{' '}
          <a
            href="https://github.com/SyedSibtainRazvi/context0"
            rel="noopener"
            target="_blank"
          >
            Star on GitHub
          </a>
        </p>
      </footer>
    </>
  );
}
