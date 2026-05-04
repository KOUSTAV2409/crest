(function () {
  var reduce = window.matchMedia('(prefers-reduced-motion: reduce)');

  function syncReducedMotion() {
    document.documentElement.classList.toggle('motion-off', reduce.matches);
  }

  syncReducedMotion();
  reduce.addEventListener('change', syncReducedMotion);

  var nav = document.querySelector('.motion-nav-shell');
  function onScroll() {
    if (!nav) return;
    nav.dataset.scrolled = window.scrollY > 10 ? 'true' : 'false';
  }

  window.addEventListener('scroll', onScroll, { passive: true });
  onScroll();

  if (reduce.matches) {
    document.querySelectorAll('.motion-rise, .motion-divider').forEach(function (el) {
      el.classList.add('is-visible');
    });
    return;
  }

  var observer = new IntersectionObserver(
    function (entries) {
      entries.forEach(function (entry) {
        if (entry.isIntersecting) {
          entry.target.classList.add('is-visible');
        }
      });
    },
    { rootMargin: '0px 0px -8% 0px', threshold: 0 },
  );

  document.querySelectorAll('.motion-rise, .motion-divider').forEach(function (el) {
    observer.observe(el);
  });
})();
