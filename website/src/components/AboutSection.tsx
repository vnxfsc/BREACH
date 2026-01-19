"use client";

import { motion } from "framer-motion";
import { useInView } from "framer-motion";
import { useRef } from "react";
import { Zap, Shield, Globe } from "lucide-react";
import Image from "next/image";

const features = [
  {
    icon: Zap,
    title: "Neural Link Technology",
    description:
      "Establish mental connections with Titans through our revolutionary capture system. Your skill determines your success.",
    gradient: "from-cyan-500 to-blue-500",
  },
  {
    icon: Shield,
    title: "True Ownership",
    description:
      "Every Titan is a unique NFT on Solana. Trade, battle, and truly own your digital assets with full on-chain verification.",
    gradient: "from-purple-500 to-pink-500",
  },
  {
    icon: Globe,
    title: "Global Hunting Grounds",
    description:
      "Hunt Titans anywhere in the world using AR technology. Breach signals appear based on real-world locations.",
    gradient: "from-orange-500 to-red-500",
  },
];

export default function AboutSection() {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-100px" });

  return (
    <section id="about" className="py-24 sm:py-32 relative overflow-hidden">
      {/* Background */}
      <div className="absolute inset-0">
        <div className="absolute inset-0 bg-gradient-to-b from-[var(--color-bg-dark)] via-[var(--color-bg-darker)] to-[var(--color-bg-dark)]" />
        <div className="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-[var(--color-primary)]/20 to-transparent" />
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
              className="inline-block text-[var(--color-primary)] text-sm font-semibold tracking-widest uppercase mb-4"
              initial={{ opacity: 0 }}
              animate={isInView ? { opacity: 1 } : {}}
              transition={{ delay: 0.2 }}
            >
              The Story
            </motion.span>
            <h2 className="text-4xl sm:text-5xl md:text-6xl font-bold mb-6">
              The <span className="text-gradient">Titans</span> Have Awakened
            </h2>
            <p className="max-w-3xl mx-auto text-[var(--color-text-secondary)] text-lg sm:text-xl leading-relaxed">
              In 2031, dimensional rifts began tearing open across the globe.
              From these Breaches emerged the Titans — massive creatures from a
              parallel dimension seeking refuge in our world.
            </p>
          </motion.div>

          {/* Story Block */}
          <motion.div
            initial={{ opacity: 0, y: 40 }}
            animate={isInView ? { opacity: 1, y: 0 } : {}}
            transition={{ duration: 0.8, delay: 0.2 }}
            className="glass-strong rounded-3xl p-8 md:p-12 mb-20 glow-border card-hover"
          >
            <div className="grid md:grid-cols-2 gap-12 items-center">
              <div>
                <h3 className="text-3xl sm:text-4xl font-bold mb-6">
                  Become a <span className="text-gradient-cyan">Linker</span>
                </h3>
                <p className="text-[var(--color-text-secondary)] mb-6 text-lg leading-relaxed">
                  A rare genetic mutation allows certain humans — known as Linkers — 
                  to establish neural connections with Titans. Through this bond, 
                  Linkers can tame these colossal beings and command them in battle.
                </p>
                <p className="text-[var(--color-text-secondary)] text-lg leading-relaxed">
                  You have awakened as a Linker. Track Breach signals, capture Titans 
                  through the Neural Link system, and build an unstoppable army. 
                  <span className="text-white font-semibold"> The hunt begins now.</span>
                </p>
              </div>
              <div className="relative">
                {/* Titan artwork */}
                <motion.div 
                  className="relative aspect-square rounded-2xl overflow-hidden"
                  whileHover={{ scale: 1.02 }}
                  transition={{ duration: 0.3 }}
                >
                  <div className="absolute inset-0 bg-gradient-to-br from-[var(--color-primary)]/20 to-[var(--color-accent)]/20" />
                  <Image 
                    src="/images/titans/hero-banner.jpg" 
                    alt="Titan emerging from dimensional rift"
                    fill
                    sizes="(max-width: 768px) 100vw, 50vw"
                    className="object-cover"
                    priority
                  />
                  {/* Overlay gradient */}
                  <div className="absolute inset-0 bg-gradient-to-t from-[var(--color-bg-dark)] via-transparent to-transparent opacity-60" />
                </motion.div>
                {/* Glow effect */}
                <div className="absolute -inset-4 bg-gradient-to-br from-[var(--color-primary)]/20 to-[var(--color-accent)]/10 rounded-3xl blur-2xl -z-10" />
              </div>
            </div>
          </motion.div>

          {/* Feature Cards */}
          <div className="grid md:grid-cols-3 gap-6">
            {features.map((feature, index) => (
              <motion.div
                key={feature.title}
                initial={{ opacity: 0, y: 30 }}
                animate={isInView ? { opacity: 1, y: 0 } : {}}
                transition={{ duration: 0.6, delay: 0.4 + index * 0.15 }}
                className="group relative glass rounded-2xl p-8 card-hover glow-border"
              >
                {/* Icon */}
                <div className={`w-14 h-14 rounded-xl bg-gradient-to-br ${feature.gradient} p-[1px] mb-6`}>
                  <div className="w-full h-full rounded-xl bg-[var(--color-bg-card)] flex items-center justify-center">
                    <feature.icon className="w-6 h-6 text-white" />
                  </div>
                </div>
                
                <h3 className="text-xl font-bold mb-3 group-hover:text-gradient transition-all duration-300">
                  {feature.title}
                </h3>
                <p className="text-[var(--color-text-secondary)] leading-relaxed">
                  {feature.description}
                </p>

                {/* Hover line */}
                <div className={`absolute bottom-0 left-0 right-0 h-[2px] bg-gradient-to-r ${feature.gradient} opacity-0 group-hover:opacity-100 transition-opacity duration-300 rounded-b-2xl`} />
              </motion.div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}
