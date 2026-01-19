"use client";

import { motion } from "framer-motion";
import { useInView } from "framer-motion";
import { useRef } from "react";
import { Map, Gamepad2, TrendingUp, Wallet, Swords, Trophy } from "lucide-react";

const features = [
  {
    icon: Map,
    title: "AR World Exploration",
    description: "Hunt Titans in augmented reality. Breach signals appear at real-world locations — parks, landmarks, cities worldwide.",
    color: "#00d4ff",
    gradient: "from-cyan-400 to-cyan-600",
  },
  {
    icon: Gamepad2,
    title: "Neural Link Capture",
    description: "Master the rhythm-based capture minigame. Sync your mind with the Titan to establish control. Higher class = greater challenge.",
    color: "#a855f7",
    gradient: "from-purple-400 to-purple-600",
  },
  {
    icon: Swords,
    title: "Strategic Combat",
    description: "Build your team of 3 Titans. PvE auto-battles for daily rewards. Turn-based PvP for competitive ranking.",
    color: "#ef4444",
    gradient: "from-red-400 to-red-600",
  },
  {
    icon: TrendingUp,
    title: "Gene System",
    description: "Each Titan has unique DNA determining its potential. Hunt for S-rank genes to create the ultimate warrior.",
    color: "#22c55e",
    gradient: "from-green-400 to-green-600",
  },
  {
    icon: Wallet,
    title: "True Ownership",
    description: "Titans are fully on-chain NFTs. Trade freely, verify authenticity, own your assets forever.",
    color: "#f59e0b",
    gradient: "from-amber-400 to-amber-600",
  },
  {
    icon: Trophy,
    title: "Play to Earn",
    description: "Earn $BREACH through battles, quests, and rankings. Compete in seasonal tournaments for massive rewards.",
    color: "#ff6b35",
    gradient: "from-orange-400 to-orange-600",
  },
];

export default function FeaturesSection() {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-100px" });

  return (
    <section id="features" className="py-24 sm:py-32 relative overflow-hidden">
      {/* Background elements */}
      <div className="absolute inset-0">
        <div className="absolute top-1/4 right-0 w-96 h-96 bg-[var(--color-primary)]/5 rounded-full blur-[120px]" />
        <div className="absolute bottom-1/4 left-0 w-96 h-96 bg-[var(--color-accent)]/5 rounded-full blur-[120px]" />
      </div>

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
              What We Offer
            </motion.span>
            <h2 className="text-4xl sm:text-5xl md:text-6xl font-bold mb-6">
              Core <span className="text-gradient">Features</span>
            </h2>
            <p className="max-w-2xl mx-auto text-[var(--color-text-secondary)] text-lg sm:text-xl">
              A next-generation gaming experience combining AR, blockchain, and strategy
            </p>
          </motion.div>

          {/* Features Grid */}
          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
            {features.map((feature, index) => (
              <motion.div
                key={feature.title}
                initial={{ opacity: 0, y: 30 }}
                animate={isInView ? { opacity: 1, y: 0 } : {}}
                transition={{ duration: 0.5, delay: 0.1 + index * 0.1 }}
                className="group relative"
              >
                <div className="relative glass rounded-2xl p-8 h-full card-hover overflow-hidden">
                  {/* Background glow on hover */}
                  <div 
                    className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-500 rounded-2xl"
                    style={{
                      background: `radial-gradient(circle at top left, ${feature.color}15 0%, transparent 50%)`,
                    }}
                  />

                  {/* Icon container */}
                  <div className="relative mb-6">
                    <div 
                      className={`w-16 h-16 rounded-2xl bg-gradient-to-br ${feature.gradient} p-[1px]`}
                    >
                      <div className="w-full h-full rounded-2xl bg-[var(--color-bg-dark)] flex items-center justify-center group-hover:bg-[var(--color-bg-card)] transition-colors duration-300">
                        <feature.icon
                          className="w-7 h-7 transition-transform duration-300 group-hover:scale-110"
                          style={{ color: feature.color }}
                        />
                      </div>
                    </div>
                    {/* Glow behind icon */}
                    <div 
                      className="absolute inset-0 blur-xl opacity-0 group-hover:opacity-50 transition-opacity duration-500"
                      style={{ backgroundColor: feature.color }}
                    />
                  </div>

                  <h3 className="relative text-xl font-bold mb-3 transition-colors duration-300 group-hover:text-white">
                    {feature.title}
                  </h3>
                  <p className="relative text-[var(--color-text-secondary)] leading-relaxed">
                    {feature.description}
                  </p>

                  {/* Bottom accent line */}
                  <div 
                    className="absolute bottom-0 left-0 h-[2px] w-0 group-hover:w-full transition-all duration-500 rounded-b-2xl"
                    style={{ background: `linear-gradient(90deg, ${feature.color}, transparent)` }}
                  />
                </div>
              </motion.div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}
