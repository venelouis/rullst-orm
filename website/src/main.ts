// Add Glassmorphism Tilt Effect to Feature Cards
document.addEventListener('DOMContentLoaded', () => {
  const cards = document.querySelectorAll<HTMLElement>('.card');

  cards.forEach(card => {
    card.addEventListener('mousemove', (e: MouseEvent) => {
      const rect = card.getBoundingClientRect();
      const x = e.clientX - rect.left; // x position within the element.
      const y = e.clientY - rect.top;  // y position within the element.
      
      const centerX = rect.width / 2;
      const centerY = rect.height / 2;
      
      const rotateX = ((y - centerY) / centerY) * -10;
      const rotateY = ((x - centerX) / centerX) * 10;
      
      card.style.setProperty('--rx', `${rotateX}deg`);
      card.style.setProperty('--ry', `${rotateY}deg`);
      card.classList.add('tilt');
    });

    card.addEventListener('mouseleave', () => {
      card.style.setProperty('--rx', `0deg`);
      card.style.setProperty('--ry', `0deg`);
      card.classList.remove('tilt');
    });
  });
});
