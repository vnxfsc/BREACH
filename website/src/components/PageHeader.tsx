"use client";

import { motion } from "framer-motion";

interface PageHeaderProps {
  title: string;
  subtitle?: string;
  gradient?: boolean;
}

export default function PageHeader({ title, subtitle, gradient = true }: PageHeaderProps) {
  return (
    <div className="pt-28 pb-16 sm:pt-36 sm:pb-20 relative overflow-hidden">
      {/* Background Effects */}
      <div className="absolute inset-0 bg-[var(--color-bg-darker)]">
        {/* Grid pattern */}
        <div
          className="absolute inset-0 opacity-[0.03]"
          style={{
            backgroundImage: `
              linear-gradient(rgba(0, 212, 255, 0.5) 1px, transparent 1px),
              linear-gradient(90deg, rgba(0, 212, 255, 0.5) 1px, transparent 1px)
            `,
            backgroundSize: "60px 60px",
          }}
        />
        
        {/* Glow orbs */}
        <div className="absolute top-0 left-1/4 w-[500px] h-[500px] bg-[var(--color-primary)]/10 rounded-full blur-[150px]" />
        <div className="absolute top-0 right-1/4 w-[400px] h-[400px] bg-[var(--color-accent)]/8 rounded-full blur-[120px]" />
        
        {/* Bottom gradient fade */}
        <div className="absolute bottom-0 left-0 right-0 h-32 bg-gradient-to-t from-[var(--color-bg-dark)] to-transparent" />
      </div>

      <div className="relative z-10 max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8 }}
        >
          <motion.h1
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.1 }}
            className="text-5xl sm:text-6xl md:text-7xl font-bold mb-6"
          >
            {gradient ? (
              <span className="text-gradient glow-text">{title}</span>
            ) : (
              title
            )}
          </motion.h1>
          {subtitle && (
            <motion.p
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.2 }}
              className="text-[var(--color-text-secondary)] text-lg sm:text-xl max-w-2xl mx-auto leading-relaxed"
            >
              {subtitle}
            </motion.p>
          )}
        </motion.div>
      </div>

      {/* Decorative elements */}
      <div className="absolute bottom-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-[var(--color-primary)]/20 to-transparent" />
    </div>
  );
}
