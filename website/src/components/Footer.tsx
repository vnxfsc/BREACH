"use client";

import Link from "next/link";
import { Github, Globe } from "lucide-react";

const socialLinks = [
  { name: "GitHub", icon: Github, href: "https://github.com/vnxfsc/BREACH" },
  { name: "Website", icon: Globe, href: "https://breach-jade.vercel.app" },
];

const footerLinks = [
  {
    title: "Product",
    links: [
      { name: "Features", href: "/#features" },
      { name: "Titans", href: "/#titans" },
      { name: "Tokenomics", href: "/#tokenomics" },
      { name: "Roadmap", href: "/#roadmap" },
    ],
  },
  {
    title: "Resources",
    links: [
      { name: "Documentation", href: "/docs" },
      { name: "Whitepaper", href: "/whitepaper" },
      { name: "FAQ", href: "/faq" },
    ],
  },
  {
    title: "Legal",
    links: [
      { name: "Privacy Policy", href: "/privacy" },
      { name: "Terms of Service", href: "/terms" },
    ],
  },
];

export default function Footer() {
  return (
    <footer className="relative bg-[var(--color-bg-darker)] border-t border-white/5">
      {/* Top gradient line */}
      <div className="absolute top-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-[var(--color-primary)]/30 to-transparent" />
      
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-16 lg:py-20">
        <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 gap-10 lg:gap-12">
          {/* Brand */}
          <div className="col-span-2 md:col-span-4 lg:col-span-2">
            <Link href="/" className="inline-block mb-6 group">
              <span className="text-3xl font-black tracking-wider text-gradient group-hover:glow-text transition-all duration-300">
                BREACH
              </span>
            </Link>
            <p className="text-[var(--color-text-muted)] text-sm mb-8 max-w-xs leading-relaxed">
              A next-generation AR monster hunting game powered by Solana. 
              Hunt Titans, build your army, dominate the world.
            </p>
            
            {/* Social Links */}
            <div className="flex gap-3">
              {socialLinks.map((social) => (
                <a
                  key={social.name}
                  href={social.href}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="w-11 h-11 glass rounded-xl flex items-center justify-center text-[var(--color-text-muted)] hover:text-[var(--color-primary)] hover:border-[var(--color-primary)]/30 transition-all duration-300 group"
                  aria-label={social.name}
                >
                  <social.icon className="w-5 h-5 group-hover:scale-110 transition-transform duration-300" />
                </a>
              ))}
            </div>
          </div>

          {/* Links */}
          {footerLinks.map((group) => (
            <div key={group.title}>
              <h3 className="text-white font-bold mb-5 text-sm uppercase tracking-wider">
                {group.title}
              </h3>
              <ul className="space-y-3">
                {group.links.map((link) => (
                  <li key={link.name}>
                    <Link
                      href={link.href}
                      className="text-[var(--color-text-muted)] hover:text-[var(--color-primary)] text-sm transition-colors duration-200"
                    >
                      {link.name}
                    </Link>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        {/* Bottom Bar */}
        <div className="mt-16 pt-8 border-t border-white/5 flex flex-col sm:flex-row items-center justify-between gap-4">
          <p className="text-[var(--color-text-muted)] text-sm">
            © {new Date().getFullYear()} BREACH. All rights reserved.
          </p>
          <div className="flex items-center gap-2 text-sm">
            <span className="text-[var(--color-text-muted)]">Built on</span>
            <span className="text-[var(--color-primary)] font-semibold">Solana</span>
            <span className="w-2 h-2 rounded-full bg-[var(--color-primary)] animate-pulse" />
          </div>
        </div>
      </div>
    </footer>
  );
}
