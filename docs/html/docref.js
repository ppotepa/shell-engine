(function () {
  function createCard() {
    const card = document.createElement('div');
    card.className = 'doc-hover-card';
    card.hidden = true;
    document.body.appendChild(card);
    return card;
  }

  function positionCard(card, event) {
    const offset = 18;
    const maxX = window.scrollX + document.documentElement.clientWidth - card.offsetWidth - 12;
    const maxY = window.scrollY + document.documentElement.clientHeight - card.offsetHeight - 12;
    const x = Math.min(event.pageX + offset, maxX);
    const y = Math.min(event.pageY + offset, maxY);
    card.style.left = Math.max(window.scrollX + 12, x) + 'px';
    card.style.top = Math.max(window.scrollY + 12, y) + 'px';
  }

  function getData() {
    if (window.__SQ_CODEMAP__) {
      return Promise.resolve(window.__SQ_CODEMAP__);
    }
    const current = document.currentScript || document.querySelector('script[src$="docref.js"]');
    const jsonPath = current && current.dataset ? current.dataset.codemapJson : null;
    if (!jsonPath) {
      return Promise.resolve(null);
    }
    return fetch(jsonPath).then(function (response) {
      return response.ok ? response.json() : null;
    }).catch(function () {
      return null;
    });
  }

  function buildLookup(data) {
    const byId = new Map();
    const byPage = new Map();
    const byModule = new Map();
    if (data && Array.isArray(data.entities)) {
      data.entities.forEach(function (entity) {
        byId.set(entity.id, entity);
        byPage.set(entity.page, entity);
      });
    }
    if (data && Array.isArray(data.modules)) {
      data.modules.forEach(function (module) {
        byId.set(module.id, module);
        byModule.set(module.page, module);
      });
    }
    return { byId: byId, byPage: byPage, byModule: byModule, raw: data };
  }

  function renderEntity(entity) {
    const summary = entity.summary || (entity.doc && entity.doc[0]) || 'No summary available from the snapshot.';
    const signature = entity.signature ? '<pre><code>' + escapeHtml(entity.signature) + '</code></pre>' : '';
    const source = entity.source_path ? '<div class="doc-hover-source"><strong>Source:</strong> <code>' + escapeHtml(entity.source_path) + (entity.source_line ? ':' + entity.source_line : '') + '</code></div>' : '';
    return '<div class="doc-hover-kind">' + escapeHtml(entity.kind || 'entity') + '</div>' +
      '<div class="doc-hover-name">' + escapeHtml(entity.name || entity.canonical_path || entity.id) + '</div>' +
      '<div class="doc-hover-summary">' + escapeHtml(summary) + '</div>' +
      signature + source;
  }

  function escapeHtml(value) {
    return String(value)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }

  function normalizeHref(anchor) {
    const href = anchor.getAttribute('href');
    if (!href || href.startsWith('#') || href.startsWith('http://') || href.startsWith('https://') || href.startsWith('mailto:')) {
      return null;
    }
    try {
      const absolute = new URL(href, window.location.href);
      const path = absolute.pathname.replace(/\\/g, '/');
      const marker = '/docs/html/';
      const index = path.lastIndexOf(marker);
      if (index >= 0) {
        return path.slice(index + marker.length);
      }
      const docsIndex = path.indexOf('/reference/');
      if (docsIndex >= 0) {
        return path.slice(docsIndex + 1);
      }
    } catch (error) {
      return null;
    }
    return null;
  }

  function resolveEntity(anchor, lookup) {
    const entityId = anchor.dataset.entityId;
    if (entityId && lookup.byId.has(entityId)) {
      return lookup.byId.get(entityId);
    }
    const page = normalizeHref(anchor);
    if (page && lookup.byPage.has(page)) {
      return lookup.byPage.get(page);
    }
    if (page && lookup.byModule.has(page)) {
      return lookup.byModule.get(page);
    }
    return null;
  }

  getData().then(function (data) {
    if (!data) {
      return;
    }
    const lookup = buildLookup(data);
    window.ShellQuestDocMap = {
      data: data,
      getEntity: function (id) { return lookup.byId.get(id) || null; },
      getSymbol: function (name) { return data.symbols && data.symbols[name] ? data.symbols[name].map(function (id) { return lookup.byId.get(id); }).filter(Boolean) : []; }
    };

    const card = createCard();
    let activeAnchor = null;

    document.querySelectorAll('a.entity-ref, a[data-entity-id]').forEach(function (anchor) {
      const entity = resolveEntity(anchor, lookup);
      if (!entity) {
        return;
      }
      anchor.classList.add('entity-ref');
      anchor.title = (entity.kind || 'entity') + ': ' + (entity.canonical_path || entity.name || entity.id);
      anchor.addEventListener('mouseenter', function (event) {
        activeAnchor = anchor;
        card.innerHTML = renderEntity(entity);
        card.hidden = false;
        positionCard(card, event);
      });
      anchor.addEventListener('mousemove', function (event) {
        if (activeAnchor === anchor && !card.hidden) {
          positionCard(card, event);
        }
      });
      anchor.addEventListener('mouseleave', function () {
        if (activeAnchor === anchor) {
          card.hidden = true;
          activeAnchor = null;
        }
      });
      anchor.addEventListener('blur', function () {
        if (activeAnchor === anchor) {
          card.hidden = true;
          activeAnchor = null;
        }
      });
    });
  });
})();
