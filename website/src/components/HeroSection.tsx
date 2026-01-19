"use client";

import { motion } from "framer-motion";
import { ChevronDown, Zap } from "lucide-react";
import { useEffect, useState } from "react";

// Particle component for floating effect
function Particle({ delay, duration, x, y, size }: { delay: number; duration: number; x: number; y: number; size: number }) {
  return (
    <motion.div
      className="absolute rounded-full"
      style={{
        left: `${x}%`,
        top: `${y}%`,
        width: size,
        height: size,
        background: `radial-gradient(circle, ${Math.random() > 0.5 ? 'var(--color-primary)' : 'var(--color-accent)'} 0%, transparent 70%)`,
      }}
      initial={{ opacity: 0, scale: 0 }}
      animate={{
        opacity: [0, 0.6, 0],
        scale: [0, 1, 0.5],
        y: [0, -100],
      }}
      transition={{
        duration,
        delay,
        repeat: Infinity,
        ease: "easeOut",
      }}
    />
  );
}

export default function HeroSection() {
  const [particles, setParticles] = useState<Array<{ id: number; delay: number; duration: number; x: number; y: number; size: number }>>([]);

  useEffect(() => {
    const newParticles = Array.from({ length: 30 }, (_, i) => ({
      id: i,
      delay: Math.random() * 5,
      duration: 3 + Math.random() * 4,
      x: Math.random() * 100,
      y: 50 + Math.random() * 50,
      size: 4 + Math.random() * 8,
    }));
    setParticles(newParticles);
  }, []);

  return (
    <section className="relative min-h-screen flex items-center justify-center overflow-hidden noise-bg">
      {/* Deep space background */}
      <div className="absolute inset-0 bg-[var(--color-bg-darker)]">
        {/* Radial gradient base */}
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,rgba(0,212,255,0.08)_0%,transparent_70%)]" />
        
        {/* Animated grid */}
        <div
          className="absolute inset-0 opacity-[0.07]"
          style={{
            backgroundImage: `
              linear-gradient(rgba(0, 212, 255, 0.3) 1px, transparent 1px),
              linear-gradient(90deg, rgba(0, 212, 255, 0.3) 1px, transparent 1px)
            `,
            backgroundSize: "60px 60px",
          }}
        />

        {/* Large glow orbs */}
        <motion.div 
          className="absolute top-1/4 left-1/4 w-[600px] h-[600px] rounded-full"
          style={{
            background: "radial-gradient(circle, rgba(0, 212, 255, 0.15) 0%, transparent 70%)",
          }}
          animate={{
            scale: [1, 1.2, 1],
            opacity: [0.3, 0.5, 0.3],
          }}
          transition={{ duration: 8, repeat: Infinity, ease: "easeInOut" }}
        />
        <motion.div 
          className="absolute bottom-1/4 right-1/4 w-[500px] h-[500px] rounded-full"
          style={{
            background: "radial-gradient(circle, rgba(255, 107, 53, 0.12) 0%, transparent 70%)",
          }}
          animate={{
            scale: [1.2, 1, 1.2],
            opacity: [0.3, 0.5, 0.3],
          }}
          transition={{ duration: 10, repeat: Infinity, ease: "easeInOut" }}
        />

        {/* Breach Rift - Central glowing crack */}
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2">
          <motion.div 
            className="relative"
            animate={{ opacity: [0.5, 0.8, 0.5] }}
            transition={{ duration: 4, repeat: Infinity }}
          >
            <div className="w-1 h-[400px] bg-gradient-to-b from-transparent via-[var(--color-primary)] to-transparent blur-sm" />
            <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[2px] h-[400px] bg-gradient-to-b from-transparent via-white to-transparent" />
          </motion.div>
        </div>

        {/* Floating particles */}
        {particles.map((particle) => (
          <Particle key={particle.id} {...particle} />
        ))}
      </div>

      {/* Content */}
      <div className="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 1 }}
        >
          {/* Badge */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3, duration: 0.6 }}
            className="inline-flex items-center gap-2 px-5 py-2.5 mb-8 rounded-full glass glow-border"
          >
            <Zap className="w-4 h-4 text-[var(--color-primary)]" />
            <span className="text-[var(--color-primary)] text-sm font-semibold tracking-wider uppercase">
              Built on Solana
            </span>
          </motion.div>

          {/* Main Title */}
          <motion.h1
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.5, duration: 0.8 }}
            className="text-6xl sm:text-7xl md:text-8xl lg:text-9xl font-black tracking-tight mb-6"
          >
            <span className="text-gradient glow-text">BREACH</span>
          </motion.h1>

          {/* Tagline */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.7, duration: 0.8 }}
            className="mb-6"
          >
            <p className="text-2xl sm:text-3xl md:text-4xl text-white/90 font-light tracking-[0.2em] uppercase">
              Hunt<span className="text-[var(--color-primary)] mx-4">•</span>
              Capture<span className="text-[var(--color-accent)] mx-4">•</span>
              Dominate
            </p>
          </motion.div>

          {/* Description */}
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.9, duration: 0.8 }}
            className="max-w-2xl mx-auto text-lg sm:text-xl text-[var(--color-text-secondary)] mb-12 leading-relaxed"
          >
            The Breach has opened. Massive Titans are emerging from dimensional rifts. 
            Become a Linker, establish neural connections, and build your army of giants.
          </motion.p>

          {/* CTA Buttons */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 1.1, duration: 0.8 }}
            className="flex flex-col sm:flex-row items-center justify-center gap-4 mb-16"
          >
            <a
              href="#waitlist"
              className="btn-primary px-10 py-4 rounded-lg text-lg"
            >
              <span>Join the Hunt</span>
            </a>
            <a
              href="#about"
              className="btn-secondary px-10 py-4 rounded-lg text-lg"
            >
              Learn More
            </a>
          </motion.div>

          {/* Stats */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 1.3, duration: 0.8 }}
            className="flex items-center justify-center gap-8 sm:gap-16"
          >
            {[
              { value: "100+", label: "Titan Species" },
              { value: "5", label: "Threat Classes" },
              { value: "∞", label: "Possibilities" },
            ].map((stat, index) => (
              <motion.div 
                key={index} 
                className="text-center"
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 1.4 + index * 0.1, duration: 0.5 }}
              >
                <div className="text-3xl sm:text-4xl md:text-5xl font-black text-gradient mb-1">
                  {stat.value}
                </div>
                <div className="text-xs sm:text-sm text-[var(--color-text-muted)] uppercase tracking-wider">
                  {stat.label}
                </div>
              </motion.div>
            ))}
          </motion.div>
        </motion.div>
      </div>

      {/* Scroll Indicator */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 2, duration: 0.8 }}
        className="absolute bottom-8 left-1/2 -translate-x-1/2"
      >
        <a 
          href="#about" 
          className="flex flex-col items-center gap-2 text-[var(--color-text-muted)] hover:text-[var(--color-primary)] transition-colors group"
        >
          <span className="text-xs uppercase tracking-widest">Scroll to explore</span>
          <motion.div
            animate={{ y: [0, 8, 0] }}
            transition={{ duration: 1.5, repeat: Infinity }}
          >
            <ChevronDown className="w-5 h-5" />
          </motion.div>
        </a>
      </motion.div>

      {/* Bottom gradient fade */}
      <div className="absolute bottom-0 left-0 right-0 h-32 bg-gradient-to-t from-[var(--color-bg-dark)] to-transparent" />
    </section>
  );
}
