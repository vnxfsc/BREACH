"use client";

import { motion } from "framer-motion";
import { useInView } from "framer-motion";
import { useRef } from "react";
import Image from "next/image";

const titanClasses = [
  {
    class: "I",
    name: "Pioneer",
    size: "15-30m",
    rarity: "Common",
    rarityPercent: "60%",
    color: "#6b7280",
    bgGradient: "from-gray-600/20 to-gray-800/20",
    description: "First to emerge from rifts. Ideal for new Linkers.",
  },
  {
    class: "II",
    name: "Hunter",
    size: "30-60m",
    rarity: "Uncommon",
    rarityPercent: "25%",
    color: "#22c55e",
    bgGradient: "from-green-600/20 to-green-800/20",
    description: "Evolved hunters with unique abilities.",
  },
  {
    class: "III",
    name: "Destroyer",
    size: "60-100m",
    rarity: "Rare",
    rarityPercent: "10%",
    color: "#3b82f6",
    bgGradient: "from-blue-600/20 to-blue-800/20",
    description: "Devastating power. Veteran Linkers only.",
  },
  {
    class: "IV",
    name: "Calamity",
    size: "100-200m",
    rarity: "Epic",
    rarityPercent: "4%",
    color: "#a855f7",
    bgGradient: "from-purple-600/20 to-purple-800/20",
    description: "Walking catastrophes. Legendary encounters.",
  },
  {
    class: "V",
    name: "Apex",
    size: "200m+",
    rarity: "Legendary",
    rarityPercent: "1%",
    color: "#f59e0b",
    bgGradient: "from-amber-600/20 to-amber-800/20",
    description: "Rulers of the Titan realm. Guild cooperation required.",
  },
];

const titanTypes = [
  { name: "Abyssal", image: "/images/titans/abyssal.jpg", color: "#0ea5e9", beats: "Volcanic" },
  { name: "Volcanic", image: "/images/titans/volcanic.jpg", color: "#ef4444", beats: "Storm" },
  { name: "Storm", image: "/images/titans/storm.jpg", color: "#a855f7", beats: "Void" },
  { name: "Void", image: "/images/titans/void.jpg", color: "#6366f1", beats: "Parasitic" },
  { name: "Parasitic", image: "/images/titans/parasitic.jpg", color: "#22c55e", beats: "Ossified" },
  { name: "Ossified", image: "/images/titans/ossified.jpg", color: "#6b7280", beats: "Abyssal" },
];

export default function TitansSection() {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-100px" });

  return (
    <section id="titans" className="py-24 sm:py-32 relative overflow-hidden">
      {/* Background */}
      <div className="absolute inset-0 bg-gradient-to-b from-[var(--color-bg-dark)] via-[var(--color-bg-darker)] to-[var(--color-bg-dark)]" />
      <div className="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-[var(--color-accent)]/20 to-transparent" />

      <div className="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div ref={ref}>
          {/* Section Header */}
          <motion.div
            initial={{ opacity: 0, y: 30 }}
            animate={isInView ? { opacity: 1, y: 0 } : {}}
            transition={{ duration: 0.8 }}
            className="text-center mb-20"
          >
            <motion.span 
              className="inline-block text-[var(--color-primary)] text-sm font-semibold tracking-widest uppercase mb-4"
              initial={{ opacity: 0 }}
              animate={isInView ? { opacity: 1 } : {}}
              transition={{ delay: 0.2 }}
            >
              Know Your Enemy
            </motion.span>
            <h2 className="text-4xl sm:text-5xl md:text-6xl font-bold mb-6">
              The <span className="text-gradient">Titans</span>
            </h2>
            <p className="max-w-2xl mx-auto text-[var(--color-text-secondary)] text-lg sm:text-xl">
              Five threat classes. Six elemental types. Infinite possibilities.
            </p>
          </motion.div>

          {/* Threat Classes */}
          <motion.div
            initial={{ opacity: 0, y: 30 }}
            animate={isInView ? { opacity: 1, y: 0 } : {}}
            transition={{ duration: 0.6, delay: 0.2 }}
            className="mb-20"
          >
            <h3 className="text-xl font-bold mb-8 text-center">
              <span className="text-[var(--color-text-muted)]">—</span>
              <span className="mx-4">Threat Classification</span>
              <span className="text-[var(--color-text-muted)]">—</span>
            </h3>
            <div className="grid sm:grid-cols-2 lg:grid-cols-5 gap-4">
              {titanClasses.map((titan, index) => (
                <motion.div
                  key={titan.class}
                  initial={{ opacity: 0, y: 20 }}
                  animate={isInView ? { opacity: 1, y: 0 } : {}}
                  transition={{ duration: 0.4, delay: 0.3 + index * 0.1 }}
                  className={`group relative glass rounded-2xl p-5 card-hover overflow-hidden`}
                >
                  {/* Background gradient */}
                  <div className={`absolute inset-0 bg-gradient-to-br ${titan.bgGradient} opacity-0 group-hover:opacity-100 transition-opacity duration-300`} />
                  
                  <div className="relative">
                    <div className="flex items-center justify-between mb-4">
                      <span
                        className="text-4xl font-black"
                        style={{ color: titan.color }}
                      >
                        {titan.class}
                      </span>
                      <span
                        className="text-xs px-3 py-1.5 rounded-full font-semibold"
                        style={{
                          backgroundColor: `${titan.color}20`,
                          color: titan.color,
                          border: `1px solid ${titan.color}40`,
                        }}
                      >
                        {titan.rarityPercent}
                      </span>
                    </div>
                    <h4 className="font-bold text-lg mb-1">{titan.name}</h4>
                    <p className="text-[var(--color-text-muted)] text-sm mb-3">
                      {titan.size}
                    </p>
                    <p className="text-[var(--color-text-secondary)] text-sm leading-relaxed">
                      {titan.description}
                    </p>
                  </div>

                  {/* Bottom accent */}
                  <div 
                    className="absolute bottom-0 left-0 right-0 h-[2px] opacity-0 group-hover:opacity-100 transition-opacity duration-300"
                    style={{ background: `linear-gradient(90deg, transparent, ${titan.color}, transparent)` }}
                  />
                </motion.div>
              ))}
            </div>
          </motion.div>

          {/* Elemental Types */}
          <motion.div
            initial={{ opacity: 0, y: 30 }}
            animate={isInView ? { opacity: 1, y: 0 } : {}}
            transition={{ duration: 0.6, delay: 0.4 }}
          >
            <h3 className="text-xl font-bold mb-8 text-center">
              <span className="text-[var(--color-text-muted)]">—</span>
              <span className="mx-4">Elemental Types</span>
              <span className="text-[var(--color-text-muted)]">—</span>
            </h3>
            <div className="glass-strong rounded-3xl p-8 md:p-10">
              <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-6 gap-6 mb-10">
                {titanTypes.map((type, index) => (
                  <motion.div
                    key={type.name}
                    initial={{ opacity: 0, scale: 0.9 }}
                    animate={isInView ? { opacity: 1, scale: 1 } : {}}
                    transition={{ duration: 0.4, delay: 0.5 + index * 0.08 }}
                    className="group text-center cursor-pointer"
                  >
                    <div className="relative w-20 h-20 mx-auto mb-3">
                      {/* Glow ring */}
                      <div 
                        className="absolute inset-0 rounded-2xl opacity-0 group-hover:opacity-100 transition-opacity duration-300 blur-lg"
                        style={{ backgroundColor: type.color }}
                      />
                      {/* Image container */}
                      <div 
                        className="relative w-full h-full rounded-2xl overflow-hidden border-2 transition-all duration-300 group-hover:scale-110"
                        style={{ borderColor: `${type.color}40` }}
                      >
                        <Image 
                          src={type.image} 
                          alt={type.name}
                          fill
                          sizes="80px"
                          className="object-cover"
                          loading="lazy"
                        />
                        {/* Overlay */}
                        <div 
                          className="absolute inset-0 opacity-0 group-hover:opacity-30 transition-opacity duration-300"
                          style={{ backgroundColor: type.color }}
                        />
                      </div>
                    </div>
                    <div
                      className="font-bold text-sm mb-1 transition-colors duration-300"
                      style={{ color: type.color }}
                    >
                      {type.name}
                    </div>
                    <div className="text-[var(--color-text-muted)] text-xs flex items-center justify-center gap-1">
                      <span>→</span>
                      <span>{type.beats}</span>
                    </div>
                  </motion.div>
                ))}
              </div>

              {/* Type Effectiveness Cycle */}
              <div className="text-center pt-6 border-t border-white/5">
                <p className="text-[var(--color-text-muted)] text-sm mb-4 uppercase tracking-wider">
                  Type Effectiveness Cycle
                </p>
                <div className="flex flex-wrap items-center justify-center gap-2 font-mono text-sm">
                  {titanTypes.map((type, index) => (
                    <span key={type.name} className="flex items-center">
                      <span style={{ color: type.color }} className="font-semibold">{type.name}</span>
                      {index < titanTypes.length - 1 && (
                        <span className="mx-2 text-[var(--color-text-muted)]">→</span>
                      )}
                    </span>
                  ))}
                  <span className="mx-2 text-[var(--color-text-muted)]">→</span>
                  <span style={{ color: titanTypes[0].color }} className="font-semibold">{titanTypes[0].name}</span>
                </div>
              </div>
            </div>
          </motion.div>
        </div>
      </div>
    </section>
  );
}
