// settings-ui/src/app.js - Vanilla tab controller and theme switcher
document.addEventListener("DOMContentLoaded", () => {
  const navItems = document.querySelectorAll(".nav-item");
  const tabPanels = document.querySelectorAll(".tab-panel");
  const themeSelect = document.getElementById("select-theme");

  navItems.forEach((item) => {
    item.addEventListener("click", () => {
      navItems.forEach((n) => {
        n.classList.remove("active");
        n.removeAttribute("aria-current");
      });
      tabPanels.forEach((p) => p.classList.remove("active"));

      item.classList.add("active");
      item.setAttribute("aria-current", "page");
      const tabId = `tab-${item.getAttribute("data-tab")}`;
      const targetPanel = document.getElementById(tabId);
      if (targetPanel) {
        targetPanel.classList.add("active");
      }
    });
  });

  if (themeSelect) {
    themeSelect.addEventListener("change", (e) => {
      document.body.setAttribute("data-theme", e.target.value);
    });
  }
});
