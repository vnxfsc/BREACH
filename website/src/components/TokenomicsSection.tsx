"use client";

import { motion } from "framer-motion";
import { useInView } from "framer-motion";
import { useRef } from "react";
import { Target, TrendingUp, Vote, Flame, Store, Coins } from "lucide-react";
import Image from "next/image";

const tokenInfo = {
  name: "$BREACH",
  totalSupply: "1,000,000,000",
  chain: "Solana",
  mintAddress: "CSH2Vz4MbgTLzB9SYJ7gBwNsyu7nKpbvEJzKQLgmmjt4",
  network: "Devnet",
};

const distribution = [
  { name: "Play-to-Earn", percent: 35, color: "#00d4ff" },
  { name: "Ecosystem", percent: 25, color: "#22c55e" },
  { name: "Team (Vested)", percent: 15, color: "#a855f7" },
  { name: "Treasury", percent: 10, color: "#f59e0b" },
  { name: "Liquidity", percent: 10, color: "#ef4444" },
  { name: "Advisors", percent: 5, color: "#ff6b35" },
];

const useCases = [
  { icon: Target, name: "Capture Costs", description: "Spend to capture Titans", color: "#00d4ff" },
  { icon: TrendingUp, name: "Upgrades", description: "Level up and evolve", color: "#22c55e" },
  { icon: Flame, name: "Fusion", description: "Combine Titans for new DNA", color: "#ef4444" },
  { icon: Store, name: "Marketplace", description: "Trade fees", color: "#f59e0b" },
  { icon: Vote, name: "Governance", description: "Vote on game decisions", color: "#a855f7" },
  { icon: Coins, name: "Staking", description: "Earn rewards and perks", color: "#ff6b35" },
];

export default function TokenomicsSection() {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-100px" });

  // Calculate total for verification
  const total = distribution.reduce((sum, item) => sum + item.percent, 0);

  return (
    <section id="tokenomics" className="py-24 sm:py-32 relative overflow-hidden">
      {/* Background */}
      <div className="absolute inset-0">
        <div className="absolute inset-0 bg-gradient-to-b from-[var(--color-bg-dark)] via-[var(--color-bg-darker)] to-[var(--color-bg-dark)]" />
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-[var(--color-primary)]/5 rounded-full blur-[150px]" />
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
              Token Economy
            </motion.span>
            <h2 className="text-4xl sm:text-5xl md:text-6xl font-bold mb-6">
              <span className="text-gradient">$BREACH</span> Token
            </h2>
            <p className="max-w-2xl mx-auto text-[var(--color-text-secondary)] text-lg sm:text-xl">
              The native token powering the BREACH ecosystem
            </p>
          </motion.div>

          <div className="grid lg:grid-cols-2 gap-8 mb-16 items-stretch">
            {/* Token Info Card */}
            <motion.div
              initial={{ opacity: 0, x: -30 }}
              animate={isInView ? { opacity: 1, x: 0 } : {}}
              transition={{ duration: 0.6, delay: 0.2 }}
              className="glass-strong rounded-3xl p-8 card-hover glow-border flex flex-col"
            >
              <div className="flex items-center mb-8">
                <div className="w-20 h-20 rounded-2xl overflow-hidden relative border-2 border-[var(--color-primary)]/30">
                  <Image 
                    src="/images/token-logo.jpg"
                    alt="$BREACH Token"
                    fill
                    className="object-cover"
                  />
                </div>
                <div className="ml-5">
                  <h3 className="text-3xl font-black text-gradient">{tokenInfo.name}</h3>
                  <p className="text-[var(--color-text-muted)] text-sm uppercase tracking-wider">Native Token</p>
                </div>
              </div>

              <div className="space-y-1 flex-1 flex flex-col justify-center">
                {[
                  { label: "Total Supply", value: tokenInfo.totalSupply },
                  { label: "Chain", value: tokenInfo.chain },
                  { label: "Network", value: tokenInfo.network },
                  { label: "Decimals", value: "9" },
                ].map((item, index) => (
                  <motion.div
                    key={item.label}
                    initial={{ opacity: 0, x: -20 }}
                    animate={isInView ? { opacity: 1, x: 0 } : {}}
                    transition={{ delay: 0.4 + index * 0.1 }}
                    className="flex justify-between items-center py-4 border-b border-white/5 last:border-0"
                  >
                    <span className="text-[var(--color-text-secondary)]">{item.label}</span>
                    <span className="font-bold text-white">{item.value}</span>
                  </motion.div>
                ))}
                {/* Token Address */}
                <motion.div
                  initial={{ opacity: 0, x: -20 }}
                  animate={isInView ? { opacity: 1, x: 0 } : {}}
                  transition={{ delay: 0.8 }}
                  className="pt-4"
                >
                  <span className="text-[var(--color-text-secondary)] text-sm">Token Address</span>
                  <a 
                    href={`https://explorer.solana.com/address/${tokenInfo.mintAddress}?cluster=devnet`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="block mt-1 font-mono text-xs text-[var(--color-primary)] hover:text-[var(--color-accent)] transition-colors truncate"
                  >
                    {tokenInfo.mintAddress}
                  </a>
                </motion.div>
              </div>
            </motion.div>

            {/* Distribution Card */}
            <motion.div
              initial={{ opacity: 0, x: 30 }}
              animate={isInView ? { opacity: 1, x: 0 } : {}}
              transition={{ duration: 0.6, delay: 0.3 }}
              className="glass-strong rounded-3xl p-8 card-hover glow-border"
            >
              <h3 className="text-xl font-bold mb-8">Distribution</h3>
              
              {/* Visual pie representation */}
              <div className="relative w-48 h-48 mx-auto mb-8">
                <svg viewBox="0 0 100 100" className="w-full h-full -rotate-90">
                  {distribution.map((item, index) => {
                    const previousPercents = distribution.slice(0, index).reduce((sum, d) => sum + d.percent, 0);
                    const circumference = 2 * Math.PI * 40;
                    const strokeDasharray = `${(item.percent / 100) * circumference} ${circumference}`;
                    const strokeDashoffset = -((previousPercents / 100) * circumference);
                    
                    return (
                      <motion.circle
                        key={item.name}
                        cx="50"
                        cy="50"
                        r="40"
                        fill="none"
                        stroke={item.color}
                        strokeWidth="12"
                        strokeDasharray={strokeDasharray}
                        strokeDashoffset={strokeDashoffset}
                        initial={{ opacity: 0 }}
                        animate={isInView ? { opacity: 1 } : {}}
                        transition={{ delay: 0.5 + index * 0.1, duration: 0.5 }}
                      />
                    );
                  })}
                </svg>
                <div className="absolute inset-0 flex items-center justify-center">
                  <div className="text-center">
                    <div className="text-2xl font-black text-white">{total}%</div>
                    <div className="text-xs text-[var(--color-text-muted)]">Total</div>
                  </div>
                </div>
              </div>

              <div className="space-y-3">
                {distribution.map((item, index) => (
                  <motion.div
                    key={item.name}
                    initial={{ opacity: 0, x: 20 }}
                    animate={isInView ? { opacity: 1, x: 0 } : {}}
                    transition={{ duration: 0.4, delay: 0.5 + index * 0.1 }}
                  >
                    <div className="flex justify-between items-center mb-1.5">
                      <div className="flex items-center gap-2">
                        <div 
                          className="w-3 h-3 rounded-full"
                          style={{ backgroundColor: item.color }}
                        />
                        <span className="text-[var(--color-text-secondary)] text-sm">
                          {item.name}
                        </span>
                      </div>
                      <span className="font-bold text-sm">{item.percent}%</span>
                    </div>
                    <div className="h-1.5 bg-white/5 rounded-full overflow-hidden">
                      <motion.div
                        initial={{ width: 0 }}
                        animate={isInView ? { width: `${item.percent}%` } : {}}
                        transition={{ duration: 0.8, delay: 0.6 + index * 0.1 }}
                        className="h-full rounded-full"
                        style={{ backgroundColor: item.color }}
                      />
                    </div>
                  </motion.div>
                ))}
              </div>
            </motion.div>
          </div>

          {/* Use Cases */}
          <motion.div
            initial={{ opacity: 0, y: 30 }}
            animate={isInView ? { opacity: 1, y: 0 } : {}}
            transition={{ duration: 0.6, delay: 0.5 }}
          >
            <h3 className="text-xl font-bold mb-8 text-center">
              <span className="text-[var(--color-text-muted)]">—</span>
              <span className="mx-4">Token Utility</span>
              <span className="text-[var(--color-text-muted)]">—</span>
            </h3>
            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
              {useCases.map((useCase, index) => (
                <motion.div
                  key={useCase.name}
                  initial={{ opacity: 0, y: 20 }}
                  animate={isInView ? { opacity: 1, y: 0 } : {}}
                  transition={{ duration: 0.4, delay: 0.6 + index * 0.08 }}
                  className="group glass rounded-2xl p-5 text-center card-hover"
                >
                  <div 
                    className="w-12 h-12 rounded-xl mx-auto mb-4 flex items-center justify-center transition-transform duration-300 group-hover:scale-110"
                    style={{ backgroundColor: `${useCase.color}15` }}
                  >
                    <useCase.icon 
                      className="w-6 h-6 transition-colors duration-300" 
                      style={{ color: useCase.color }}
                    />
                  </div>
                  <h4 className="font-bold text-sm mb-1 group-hover:text-white transition-colors">{useCase.name}</h4>
                  <p className="text-[var(--color-text-muted)] text-xs leading-relaxed">
                    {useCase.description}
                  </p>
                </motion.div>
              ))}
            </div>
          </motion.div>
        </div>
      </div>
    </section>
  );
}
