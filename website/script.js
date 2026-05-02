// ── Nav scroll effect
const nav = document.getElementById('nav');
window.addEventListener('scroll', () => {
  nav.classList.toggle('scrolled', window.scrollY > 20);
});

// ── Copy code button
function copyCode(btn) {
  const pre = btn.closest('.code-block').querySelector('pre');
  const text = pre.textContent.trim();
  navigator.clipboard.writeText(text).then(() => {
    btn.textContent = 'Copied!';
    btn.classList.add('copied');
    setTimeout(() => {
      btn.textContent = 'Copy';
      btn.classList.remove('copied');
    }, 2000);
  });
}

// ── Animate elements in on scroll
const observer = new IntersectionObserver((entries) => {
  entries.forEach(entry => {
    if (entry.isIntersecting) {
      entry.target.style.opacity = '1';
      entry.target.style.transform = 'translateY(0)';
    }
  });
}, { threshold: 0.1 });

document.querySelectorAll('.feature-card, .roadmap-col, .install-step, .privacy-table-wrap').forEach(el => {
  el.style.opacity = '0';
  el.style.transform = 'translateY(24px)';
  el.style.transition = 'opacity 0.5s ease, transform 0.5s ease';
  observer.observe(el);
});

// ── Typewriter effect in hero demo bars
const demos = document.querySelectorAll('.demo-cursor');
demos.forEach(el => {
  const finalText = el.textContent;
  el.textContent = '';
  let i = 0;
  const type = () => {
    if (i < finalText.length) {
      el.textContent += finalText[i++];
      setTimeout(type, 50 + Math.random() * 30);
    }
  };
  // Start after a short delay so page has rendered
  setTimeout(type, 800 + Math.random() * 400);
});
