"use client";

import { useState } from "react";
import { motion } from "framer-motion";
import { useInView } from "framer-motion";
import { useRef } from "react";
import { Mail, Check, Loader2, Sparkles, Users, Gift } from "lucide-react";

export default function WaitlistSection() {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-100px" });
  const [email, setEmail] = useState("");
  const [status, setStatus] = useState<"idle" | "loading" | "success" | "error">("idle");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!email) return;

    setStatus("loading");
    await new Promise((resolve) => setTimeout(resolve, 1500));
    setStatus("success");
    setEmail("");
    setTimeout(() => setStatus("idle"), 3000);
  };

  return (
    <section id="waitlist" className="py-24 sm:py-32 relative overflow-hidden">
      {/* Background effects */}
      <div className="absolute inset-0">
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[800px] h-[800px] bg-[var(--color-primary)]/5 rounded-full blur-[150px]" />
      </div>

      <div className="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div ref={ref}>
          <motion.div
            initial={{ opacity: 0, y: 30 }}
            animate={isInView ? { opacity: 1, y: 0 } : {}}
            transition={{ duration: 0.8 }}
            className="relative"
          >
            {/* Main card */}
            <div className="relative glass-strong rounded-3xl p-10 md:p-16 overflow-hidden">
              {/* Decorative corner accents */}
              <div className="absolute top-0 left-0 w-32 h-32">
                <div className="absolute top-4 left-4 w-16 h-[1px] bg-gradient-to-r from-[var(--color-primary)] to-transparent" />
                <div className="absolute top-4 left-4 w-[1px] h-16 bg-gradient-to-b from-[var(--color-primary)] to-transparent" />
              </div>
              <div className="absolute top-0 right-0 w-32 h-32">
                <div className="absolute top-4 right-4 w-16 h-[1px] bg-gradient-to-l from-[var(--color-accent)] to-transparent" />
                <div className="absolute top-4 right-4 w-[1px] h-16 bg-gradient-to-b from-[var(--color-accent)] to-transparent" />
              </div>
              <div className="absolute bottom-0 left-0 w-32 h-32">
                <div className="absolute bottom-4 left-4 w-16 h-[1px] bg-gradient-to-r from-[var(--color-primary)] to-transparent" />
                <div className="absolute bottom-4 left-4 w-[1px] h-16 bg-gradient-to-t from-[var(--color-primary)] to-transparent" />
              </div>
              <div className="absolute bottom-0 right-0 w-32 h-32">
                <div className="absolute bottom-4 right-4 w-16 h-[1px] bg-gradient-to-l from-[var(--color-accent)] to-transparent" />
                <div className="absolute bottom-4 right-4 w-[1px] h-16 bg-gradient-to-t from-[var(--color-accent)] to-transparent" />
              </div>

              {/* Animated background gradient */}
              <div className="absolute inset-0 opacity-30">
                <motion.div
                  className="absolute top-0 right-0 w-96 h-96 bg-[var(--color-primary)]/20 rounded-full blur-[100px]"
                  animate={{
                    x: [0, 50, 0],
                    y: [0, 30, 0],
                  }}
                  transition={{ duration: 10, repeat: Infinity, ease: "easeInOut" }}
                />
                <motion.div
                  className="absolute bottom-0 left-0 w-96 h-96 bg-[var(--color-accent)]/20 rounded-full blur-[100px]"
                  animate={{
                    x: [0, -50, 0],
                    y: [0, -30, 0],
                  }}
                  transition={{ duration: 12, repeat: Infinity, ease: "easeInOut" }}
                />
              </div>

              <div className="relative z-10 max-w-2xl mx-auto text-center">
                {/* Badge */}
                <motion.div
                  initial={{ opacity: 0, scale: 0.9 }}
                  animate={isInView ? { opacity: 1, scale: 1 } : {}}
                  transition={{ duration: 0.5, delay: 0.2 }}
                  className="inline-flex items-center gap-2 px-5 py-2.5 mb-8 rounded-full glass"
                >
                  <Sparkles className="w-4 h-4 text-[var(--color-accent)]" />
                  <span className="text-[var(--color-accent)] text-sm font-semibold uppercase tracking-wider">
                    Early Access
                  </span>
                </motion.div>

                <h2 className="text-4xl sm:text-5xl md:text-6xl font-bold mb-6">
                  Join the <span className="text-gradient">Hunt</span>
                </h2>

                <p className="text-[var(--color-text-secondary)] text-lg sm:text-xl mb-10 leading-relaxed">
                  Be among the first Linkers to enter the Breach. 
                  Get exclusive early access, rewards, and the chance to shape the game.
                </p>

                {/* Email Form */}
                <form onSubmit={handleSubmit} className="flex flex-col sm:flex-row gap-4 max-w-lg mx-auto mb-10">
                  <div className="relative flex-1">
                    <Mail className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-[var(--color-text-muted)]" />
                    <input
                      type="email"
                      value={email}
                      onChange={(e) => setEmail(e.target.value)}
                      placeholder="Enter your email"
                      className="w-full pl-12 pr-4 py-4 glass rounded-xl text-white placeholder-[var(--color-text-muted)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]/50 transition-all duration-300"
                      disabled={status === "loading" || status === "success"}
                    />
                  </div>
                  <button
                    type="submit"
                    disabled={status === "loading" || status === "success"}
                    className="btn-primary px-8 py-4 rounded-xl flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed min-w-[160px]"
                  >
                    <span className="flex items-center gap-2">
                      {status === "loading" && <Loader2 className="w-5 h-5 animate-spin" />}
                      {status === "success" && <Check className="w-5 h-5" />}
                      {status === "idle" && "Join Waitlist"}
                      {status === "loading" && "Joining..."}
                      {status === "success" && "Joined!"}
                    </span>
                  </button>
                </form>

                {status === "success" && (
                  <motion.p
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    className="text-[var(--color-primary)] text-sm mb-8"
                  >
                    ✓ Welcome to the hunt! Check your email for confirmation.
                  </motion.p>
                )}

                {/* Benefits */}
                <div className="grid sm:grid-cols-3 gap-6 pt-8 border-t border-white/5">
                  {[
                    { icon: Users, label: "10,000+ hunters waiting", value: "Community" },
                    { icon: Gift, label: "Exclusive early rewards", value: "Rewards" },
                    { icon: Sparkles, label: "Shape the game", value: "Influence" },
                  ].map((item, index) => (
                    <motion.div
                      key={item.label}
                      initial={{ opacity: 0, y: 20 }}
                      animate={isInView ? { opacity: 1, y: 0 } : {}}
                      transition={{ delay: 0.6 + index * 0.1 }}
                      className="text-center"
                    >
                      <item.icon className="w-6 h-6 text-[var(--color-primary)] mx-auto mb-2" />
                      <p className="text-[var(--color-text-secondary)] text-sm">{item.label}</p>
                    </motion.div>
                  ))}
                </div>
              </div>
            </div>
          </motion.div>
        </div>
      </div>
    </section>
  );
}
