"use client";

import { motion } from "framer-motion";
import { useInView } from "framer-motion";
import { useRef } from "react";
import { Check, Clock, Circle, Rocket, Gamepad2, Coins, Globe } from "lucide-react";

const roadmapPhases = [
  {
    phase: "Phase 1",
    title: "Foundation",
    icon: Rocket,
    status: "in-progress",
    timeline: "Q1 2026",
    color: "#00d4ff",
    items: [
      { text: "Core smart contracts development", done: true },
      { text: "Titan attribute system design", done: true },
      { text: "Official website launch", done: true },
      { text: "Community building", done: false },
    ],
  },
  {
    phase: "Phase 2",
    title: "Alpha Launch",
    icon: Gamepad2,
    status: "upcoming",
    timeline: "Q2 2026",
    color: "#a855f7",
    items: [
      { text: "Closed alpha testing", done: false },
      { text: "Neural Link capture system", done: false },
      { text: "Basic PvE battles", done: false },
      { text: "Initial Titan collection", done: false },
    ],
  },
  {
    phase: "Phase 3",
    title: "Beta & Economy",
    icon: Coins,
    status: "upcoming",
    timeline: "Q3 2026",
    color: "#22c55e",
    items: [
      { text: "Public beta release", done: false },
      { text: "$BREACH token launch", done: false },
      { text: "Marketplace integration", done: false },
      { text: "Guild system", done: false },
    ],
  },
  {
    phase: "Phase 4",
    title: "Full Launch",
    icon: Globe,
    status: "upcoming",
    timeline: "Q4 2026",
    color: "#ff6b35",
    items: [
      { text: "Global launch", done: false },
      { text: "PvP ranked battles", done: false },
      { text: "World Boss events", done: false },
      { text: "Seasonal content", done: false },
    ],
  },
];

export default function RoadmapSection() {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-100px" });

  const getStatusIcon = (status: string, done?: boolean) => {
    if (done) return <Check className="w-4 h-4" />;
    if (status === "in-progress") return <Clock className="w-4 h-4" />;
    return <Circle className="w-4 h-4" />;
  };

  return (
    <section id="roadmap" className="py-24 sm:py-32 relative overflow-hidden">
      {/* Background */}
      <div className="absolute inset-0 bg-gradient-to-b from-[var(--color-bg-dark)] via-[var(--color-bg-darker)] to-[var(--color-bg-dark)]" />
      <div className="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-[var(--color-primary)]/20 to-transparent" />

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
              className="inline-block text-[var(--color-accent)] text-sm font-semibold tracking-widest uppercase mb-4"
              initial={{ opacity: 0 }}
              animate={isInView ? { opacity: 1 } : {}}
              transition={{ delay: 0.2 }}
            >
              Our Journey
            </motion.span>
            <h2 className="text-4xl sm:text-5xl md:text-6xl font-bold mb-6">
              <span className="text-gradient">Roadmap</span>
            </h2>
            <p className="max-w-2xl mx-auto text-[var(--color-text-secondary)] text-lg sm:text-xl">
              Our journey to launch the ultimate Titan hunting experience
            </p>
          </motion.div>

          {/* Timeline */}
          <div className="relative">
            {/* Connection Line - Desktop */}
            <div className="hidden lg:block absolute top-24 left-0 right-0 h-[2px]">
              <div className="h-full bg-gradient-to-r from-[var(--color-primary)]/50 via-[var(--color-accent)]/30 to-transparent" />
            </div>

            {/* Phases Grid */}
            <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
              {roadmapPhases.map((phase, index) => (
                <motion.div
                  key={phase.phase}
                  initial={{ opacity: 0, y: 30 }}
                  animate={isInView ? { opacity: 1, y: 0 } : {}}
                  transition={{ duration: 0.6, delay: 0.2 + index * 0.15 }}
                  className="relative"
                >
                  {/* Timeline node - Desktop */}
                  <div className="hidden lg:flex justify-center mb-8">
                    <motion.div
                      className="relative"
                      initial={{ scale: 0 }}
                      animate={isInView ? { scale: 1 } : {}}
                      transition={{ delay: 0.4 + index * 0.15, type: "spring" }}
                    >
                      <div 
                        className="w-12 h-12 rounded-xl flex items-center justify-center"
                        style={{ 
                          backgroundColor: `${phase.color}20`,
                          border: `2px solid ${phase.color}`,
                          boxShadow: phase.status === "in-progress" ? `0 0 20px ${phase.color}40` : "none"
                        }}
                      >
                        <phase.icon className="w-5 h-5" style={{ color: phase.color }} />
                      </div>
                      {phase.status === "in-progress" && (
                        <div 
                          className="absolute inset-0 rounded-xl animate-ping opacity-30"
                          style={{ backgroundColor: phase.color }}
                        />
                      )}
                    </motion.div>
                  </div>

                  {/* Card */}
                  <div
                    className={`glass rounded-2xl p-6 h-full card-hover relative overflow-hidden ${
                      phase.status === "in-progress" ? "ring-1 ring-[var(--color-primary)]/30" : ""
                    }`}
                    style={{
                      borderColor: phase.status === "in-progress" ? `${phase.color}40` : "transparent"
                    }}
                  >
                    {/* Status indicator glow */}
                    {phase.status === "in-progress" && (
                      <div 
                        className="absolute top-0 left-0 right-0 h-1"
                        style={{ background: `linear-gradient(90deg, transparent, ${phase.color}, transparent)` }}
                      />
                    )}

                    {/* Mobile icon */}
                    <div className="lg:hidden flex items-center gap-3 mb-4">
                      <div 
                        className="w-10 h-10 rounded-lg flex items-center justify-center"
                        style={{ backgroundColor: `${phase.color}20` }}
                      >
                        <phase.icon className="w-5 h-5" style={{ color: phase.color }} />
                      </div>
                      <span 
                        className="text-xs px-3 py-1 rounded-full font-semibold"
                        style={{ 
                          backgroundColor: `${phase.color}20`,
                          color: phase.color 
                        }}
                      >
                        {phase.timeline}
                      </span>
                    </div>

                    {/* Desktop timeline badge */}
                    <div className="hidden lg:block mb-4">
                      <span 
                        className="text-xs px-3 py-1.5 rounded-full font-semibold"
                        style={{ 
                          backgroundColor: `${phase.color}20`,
                          color: phase.color 
                        }}
                      >
                        {phase.timeline}
                      </span>
                    </div>

                    <h3 className="text-xl font-bold mb-4">{phase.title}</h3>

                    <ul className="space-y-3">
                      {phase.items.map((item, itemIndex) => (
                        <motion.li
                          key={itemIndex}
                          initial={{ opacity: 0, x: -10 }}
                          animate={isInView ? { opacity: 1, x: 0 } : {}}
                          transition={{ delay: 0.5 + index * 0.15 + itemIndex * 0.05 }}
                          className="flex items-start gap-3"
                        >
                          <div 
                            className={`w-5 h-5 rounded-full flex items-center justify-center flex-shrink-0 mt-0.5 ${
                              item.done 
                                ? "bg-[var(--color-primary)]/20 text-[var(--color-primary)]" 
                                : "bg-white/5 text-[var(--color-text-muted)]"
                            }`}
                          >
                            {getStatusIcon(phase.status, item.done)}
                          </div>
                          <span
                            className={`text-sm leading-relaxed ${
                              item.done
                                ? "text-[var(--color-text-secondary)]"
                                : "text-[var(--color-text-muted)]"
                            }`}
                          >
                            {item.text}
                          </span>
                        </motion.li>
                      ))}
                    </ul>
                  </div>
                </motion.div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
