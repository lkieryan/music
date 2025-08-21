import type { FC } from "react";

/**
 * GradientBackground
 * Two fixed full-screen layers to support cross-fade between
 * current and previous gradients using CSS variables.
 * This component preserves the legacy visual 1:1 and accepts no props.
 */
const GradientBackground: FC = () => {
  return (
    <>
      <div
        className="fixed inset-0 pointer-events-none z-0 transition-opacity duration-300"
        style={{
          background: 'var(--app-background-gradient, var(--main-browser-background, linear-gradient(135deg, #667eea 0%, #764ba2 100%)))',
          opacity: 'var(--app-background-opacity, var(--background-opacity, 1))'
        }}
      />
      <div
        className="fixed inset-0 pointer-events-none z-0 transition-opacity duration-300"
        style={{
          background: 'var(--app-background-gradient-old, var(--main-browser-background-old, linear-gradient(135deg, #667eea 0%, #764ba2 100%)))',
          opacity: 'calc(1 - var(--app-background-opacity, var(--background-opacity, 1)))'
        }}
      />
    </>
  );
};

export default GradientBackground;
