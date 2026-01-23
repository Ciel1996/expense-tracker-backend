"use client";

import React, { PropsWithChildren, useEffect, useLayoutEffect, useRef, useState } from "react";

/**
 * Collapse component that animates height between 0 and its content height.
 * - Keeps content mounted to allow smooth animation back and forth.
 * - Sets height: auto after expansion completes to accommodate dynamic content.
 */
export function Collapse({ isOpen, duration = 200, animated = false, children, className = "" }: PropsWithChildren<{ isOpen: boolean; duration?: number; animated?: boolean; className?: string }>) {
  const ref = useRef<HTMLDivElement>(null);
  const [inited, setInited] = useState(false);

  // Initialize collapsed height without animation on first render
  useEffect(() => {
    setInited(true);
  }, []);

  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;

    // If animations are disabled, set height immediately
    if (!animated) {
      el.style.transition = "";
      el.style.height = isOpen ? "auto" : "0px";
      return;
    }

    const contentHeight = el.scrollHeight;

    if (isOpen) {
      // from current numeric height (0 or pixel) to contentHeight then to auto after transition
      el.style.transition = inited ? `height ${duration}ms ease` : "";
      // If height is 'auto', set it first to pixel height to let transition work
      const current = el.style.height;
      if (current === "auto") {
        el.style.height = contentHeight + "px";
      }
      requestAnimationFrame(() => {
        el.style.height = contentHeight + "px";
      });
      const onEnd = () => {
        el.style.transition = "";
        el.style.height = "auto";
        el.removeEventListener("transitionend", onEnd);
      };
      el.addEventListener("transitionend", onEnd);
    } else {
      // collapse: from current height (auto or px) to 0
      el.style.transition = inited ? `height ${duration}ms ease` : "";
      // if 'auto', set to pixel value before collapsing
      const currentHeight = el.getBoundingClientRect().height;
      el.style.height = currentHeight + "px";
      requestAnimationFrame(() => {
        el.style.height = "0px";
      });
    }
  }, [isOpen, duration, inited, animated]);

  return (
    <div
      ref={ref}
      className={className}
      style={{ overflow: "hidden", height: isOpen ? undefined : 0 }}
      aria-hidden={!isOpen}
    >
      {/* We keep children always mounted so the measurement is available */}
      <div>{children}</div>
    </div>
  );
}
