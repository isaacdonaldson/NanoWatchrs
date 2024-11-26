const DARK_THEME_TEXT = 'DARK';
const LIGHT_THEME_TEXT = 'LIGHT';

// Theme toggle functionality
const themeToggle = document.querySelector('.theme-toggle');
const prefersDarkScheme = window.matchMedia('(prefers-color-scheme: dark)');

// Function to set the theme
const setTheme = (theme) => {
  document.documentElement.setAttribute('data-theme', theme);
  localStorage.setItem('theme', theme);

  // Update toggle button text
  const toggleText = document.querySelector('.theme-toggle-text');
  toggleText.textContent = theme === 'dark' ? LIGHT_THEME_TEXT : DARK_THEME_TEXT;
};

// Initialize theme
const savedTheme = localStorage.getItem('theme');
if (savedTheme) {
  setTheme(savedTheme);
} else {
  setTheme(prefersDarkScheme.matches ? 'dark' : 'light');
}

// Toggle theme when button is clicked
themeToggle.addEventListener('click', () => {
  const currentTheme = document.documentElement.getAttribute('data-theme');
  setTheme(currentTheme === 'dark' ? 'light' : 'dark');
});

// Listen for system theme changes
prefersDarkScheme.addEventListener('change', (e) => {
  if (!localStorage.getItem('theme')) {
    setTheme(e.matches ? 'dark' : 'light');
  }
});
