const menuToggle = document.querySelector('.menu-toggle');
const topnav = document.querySelector('#topnav');

if (menuToggle && topnav) {
  menuToggle.addEventListener('click', () => {
    const isOpen = topnav.classList.toggle('open');
    menuToggle.setAttribute('aria-expanded', String(isOpen));
  });

  topnav.querySelectorAll('a').forEach((link) => {
    link.addEventListener('click', () => {
      topnav.classList.remove('open');
      menuToggle.setAttribute('aria-expanded', 'false');
    });
  });
}

const copyButtons = document.querySelectorAll('.copy-btn[data-copy-target]');

copyButtons.forEach((button) => {
  button.addEventListener('click', async () => {
    const targetId = button.getAttribute('data-copy-target');
    const codeEl = targetId ? document.getElementById(targetId) : null;
    if (!codeEl) return;

    const originalLabel = button.textContent;

    try {
      await navigator.clipboard.writeText(codeEl.innerText.trim());
      button.textContent = 'Copied';
      button.setAttribute('data-copied', 'true');
    } catch (_err) {
      button.textContent = 'Failed';
      button.setAttribute('data-copied', 'false');
    }

    window.setTimeout(() => {
      button.textContent = originalLabel;
      button.removeAttribute('data-copied');
    }, 1100);
  });
});

const tabButtons = document.querySelectorAll('.tab-btn');
const tabPanels = document.querySelectorAll('.tab-panel');

function setActiveTab(name) {
  tabButtons.forEach((button) => {
    const active = button.getAttribute('data-tab') === name;
    button.classList.toggle('active', active);
    button.setAttribute('aria-selected', String(active));
  });

  tabPanels.forEach((panel) => {
    panel.classList.toggle('active', panel.getAttribute('data-panel') === name);
  });
}

tabButtons.forEach((button) => {
  button.addEventListener('click', () => {
    const tabName = button.getAttribute('data-tab');
    if (tabName) setActiveTab(tabName);
  });
});

const revealEls = document.querySelectorAll('.reveal');
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

const sectionIds = ['what', 'quickstart', 'commands', 'mcp'];
const navLinks = Array.from(document.querySelectorAll('.topnav a'));

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
