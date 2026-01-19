import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import PageHeader from "@/components/PageHeader";
import { Metadata } from "next";

export const metadata: Metadata = {
  title: "Privacy Policy - BREACH",
  description: "BREACH privacy policy explaining how we collect, use, and protect your data.",
};

const sections = [
  {
    title: "1. Introduction",
    content: `This Privacy Policy explains how BREACH ("we", "us", or "our") collects, uses, 
    discloses, and protects your personal information when you use our mobile application 
    and related services (collectively, the "Service"). By using our Service, you agree to 
    the collection and use of information in accordance with this policy.`,
  },
  {
    title: "2. Information We Collect",
    subsections: [
      {
        subtitle: "2.1 Account Information",
        text: `When you create an account, we collect your wallet address and any profile 
        information you choose to provide (username, avatar). We do not have access to your 
        wallet's private keys.`,
      },
      {
        subtitle: "2.2 Location Data",
        text: `Our AR features require access to your device's location. We collect location 
        data only while the app is active to display nearby Breach signals and enable gameplay. 
        Location history is not stored on our servers beyond 24 hours.`,
      },
      {
        subtitle: "2.3 Device Information",
        text: `We collect device identifiers, operating system version, and hardware 
        specifications to optimize performance and provide technical support.`,
      },
      {
        subtitle: "2.4 Gameplay Data",
        text: `We collect information about your in-game activities, including Titans captured, 
        battles participated, transactions made, and achievements earned. This data is used 
        to improve game balance and provide personalized experiences.`,
      },
      {
        subtitle: "2.5 Analytics",
        text: `We use analytics tools to understand how users interact with our Service. This 
        includes session duration, features used, and crash reports. This data is aggregated 
        and anonymized.`,
      },
    ],
  },
  {
    title: "3. How We Use Your Information",
    content: `We use collected information to:
    
    • Provide and maintain the Service
    • Process in-game transactions and verify NFT ownership
    • Display location-based content (Breach signals)
    • Improve game mechanics and user experience
    • Communicate updates, promotions, and support
    • Detect and prevent fraud, cheating, and abuse
    • Comply with legal obligations`,
  },
  {
    title: "4. Blockchain Data",
    content: `BREACH operates on the Solana blockchain. Your Titans (NFTs), token transactions, 
    and wallet activity are recorded on the public blockchain. This data is inherently public 
    and immutable. We cannot delete or modify blockchain records. Your wallet address may be 
    visible to other users in leaderboards and marketplaces.`,
  },
  {
    title: "5. Data Sharing",
    subsections: [
      {
        subtitle: "5.1 Third-Party Services",
        text: `We may share data with service providers who assist in operating our Service 
        (hosting, analytics, customer support). These providers are contractually bound to 
        protect your information.`,
      },
      {
        subtitle: "5.2 Legal Requirements",
        text: `We may disclose information if required by law, court order, or government 
        request, or to protect our rights, users' safety, or the public.`,
      },
      {
        subtitle: "5.3 Business Transfers",
        text: `In the event of a merger, acquisition, or asset sale, user data may be 
        transferred. We will notify users before data becomes subject to a different 
        privacy policy.`,
      },
    ],
  },
  {
    title: "6. Data Security",
    content: `We implement industry-standard security measures including encryption, secure 
    servers, and access controls. However, no method of transmission over the internet is 
    100% secure. We cannot guarantee absolute security but continuously work to protect 
    your information.`,
  },
  {
    title: "7. Data Retention",
    content: `We retain personal data for as long as your account is active or as needed to 
    provide services. Location data is deleted after 24 hours. Analytics data is retained 
    in anonymized form indefinitely. You may request account deletion at any time.`,
  },
  {
    title: "8. Your Rights",
    content: `Depending on your jurisdiction, you may have rights to:
    
    • Access your personal data
    • Correct inaccurate data
    • Delete your account and associated data
    • Export your data in a portable format
    • Opt out of marketing communications
    • Restrict certain data processing
    
    To exercise these rights, contact us at privacy@breach.gg.`,
  },
  {
    title: "9. Children's Privacy",
    content: `Our Service is not intended for users under 13 years of age (or the applicable 
    age of consent in your jurisdiction). We do not knowingly collect data from children. 
    If you believe a child has provided us with personal information, please contact us 
    immediately.`,
  },
  {
    title: "10. International Transfers",
    content: `Your data may be transferred to and processed in countries other than your 
    country of residence. We ensure appropriate safeguards are in place for such transfers 
    in compliance with applicable laws.`,
  },
  {
    title: "11. Changes to This Policy",
    content: `We may update this Privacy Policy from time to time. We will notify you of 
    significant changes via in-app notification. Continued use of the Service 
    after changes constitutes acceptance of the revised policy.`,
  },
  {
    title: "12. Contact Us",
    content: `If you have questions about this Privacy Policy or our data practices, 
    please contact us at:
    
    GitHub: github.com/vnxfsc/BREACH/issues
    Website: breach-jade.vercel.app`,
  },
];

export default function PrivacyPage() {
  return (
    <main className="min-h-screen bg-[var(--color-bg-dark)]">
      <Navbar />
      <PageHeader 
        title="Privacy Policy" 
        subtitle="How we handle your data"
      />
      
      <div className="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8 pb-24">
        <div className="bg-[var(--color-bg-card)] border border-white/5 rounded-xl p-6 md:p-8 mb-8">
          <p className="text-[var(--color-text-muted)] text-sm">
            Last updated: January 2026
          </p>
        </div>

        <div className="space-y-8">
          {sections.map((section) => (
            <section key={section.title}>
              <h2 className="text-xl font-bold mb-4">{section.title}</h2>
              
              {section.content && (
                <p className="text-[var(--color-text-secondary)] text-sm leading-relaxed whitespace-pre-line">
                  {section.content}
                </p>
              )}
              
              {section.subsections && (
                <div className="space-y-4">
                  {section.subsections.map((sub) => (
                    <div key={sub.subtitle}>
                      <h3 className="text-base font-semibold mb-2 text-[var(--color-text-primary)]">
                        {sub.subtitle}
                      </h3>
                      <p className="text-[var(--color-text-secondary)] text-sm leading-relaxed">
                        {sub.text}
                      </p>
                    </div>
                  ))}
                </div>
              )}
            </section>
          ))}
        </div>
      </div>

      <Footer />
    </main>
  );
}
