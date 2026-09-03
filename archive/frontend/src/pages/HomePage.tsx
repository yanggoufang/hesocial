import { motion } from 'framer-motion'
import { Link } from 'react-router-dom'
import { Crown, Shield, Users, Calendar, Star, ArrowRight } from 'lucide-react'

const HomePage = () => {
  const features = [
    {
      icon: Crown,
      title: '頂級會員制',
      description: '嚴格篩選的高淨值會員，確保最優質的社交圈層'
    },
    {
      icon: Shield,
      title: '隱私保障',
      description: '企業級加密技術，絕對保護您的個人隱私與資料安全'
    },
    {
      icon: Users,
      title: 'AI智能配對',
      description: '基於興趣、背景和偏好的精準社交推薦系統'
    },
    {
      icon: Calendar,
      title: '獨家活動',
      description: '精心策劃的高端社交活動，從私人晚宴到豪華遊艇派對'
    }
  ]

  const membershipTiers = [
    {
      name: 'Platinum',
      price: 'NT$50,000',
      period: '/年',
      features: [
        '參與精選社交活動',
        '基本身份驗證',
        '標準客服支援',
        '月度活動推薦'
      ],
      popular: false
    },
    {
      name: 'Diamond',
      price: 'NT$120,000',
      period: '/年',
      features: [
        '所有 Platinum 權益',
        'VIP 活動優先預訂',
        '專屬社交顧問',
        '私人活動邀請',
        '高端場地折扣'
      ],
      popular: true
    },
    {
      name: 'Black Card',
      price: '邀請制',
      period: '',
      features: [
        '所有 Diamond 權益',
        '獨家 VVIP 活動',
        '24/7 禮賓服務',
        '客製化活動規劃',
        '全球合作夥伴特權'
      ],
      popular: false
    }
  ]

  return (
    <div className="min-h-screen">
      <section className="relative h-screen flex items-center justify-center overflow-hidden">
        <div className="absolute inset-0 luxury-gradient"></div>
        <div className="absolute inset-0 bg-luxury-midnight-black/50"></div>
        
        <motion.div
          initial={{ opacity: 0, y: 50 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 1 }}
          className="relative z-10 text-center max-w-4xl mx-auto px-4"
        >
          <h1 className="text-6xl md:text-8xl font-luxury font-bold text-luxury-gold mb-6">
            HeSocial
          </h1>
          <p className="text-xl md:text-2xl text-luxury-platinum mb-8 leading-relaxed">
            專為高淨值人士打造的頂級社交平台
            <br />
            在奢華環境中遇見志同道合的菁英
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Link to="/register" className="luxury-button text-lg px-12 py-4">
              申請加入
            </Link>
            <Link to="/events" className="luxury-button-outline text-lg px-12 py-4">
              探索活動
            </Link>
          </div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 1.5, duration: 1 }}
          className="absolute bottom-10 left-1/2 transform -translate-x-1/2"
        >
          <div className="w-6 h-10 border-2 border-luxury-gold rounded-full flex justify-center">
            <div className="w-1 h-3 bg-luxury-gold rounded-full mt-2 animate-bounce"></div>
          </div>
        </motion.div>
      </section>

      <section className="py-20 bg-luxury-midnight-black">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 50 }}
            whileInView={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8 }}
            viewport={{ once: true }}
            className="text-center mb-16"
          >
            <h2 className="text-4xl md:text-5xl font-luxury font-bold text-luxury-gold mb-6">
              為什麼選擇 HeSocial
            </h2>
            <p className="text-xl text-luxury-platinum/80 max-w-3xl mx-auto">
              我們致力於為台灣最優秀的企業家、投資人和專業人士提供最高品質的社交體驗
            </p>
          </motion.div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8">
            {features.map((feature, index) => (
              <motion.div
                key={feature.title}
                initial={{ opacity: 0, y: 50 }}
                whileInView={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.8, delay: index * 0.2 }}
                viewport={{ once: true }}
                className="luxury-glass p-8 rounded-2xl hover:bg-luxury-gold/5 transition-all duration-300 group"
              >
                <feature.icon className="h-12 w-12 text-luxury-gold mb-6 group-hover:scale-110 transition-transform duration-300" />
                <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-4">
                  {feature.title}
                </h3>
                <p className="text-luxury-platinum/80 leading-relaxed">
                  {feature.description}
                </p>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      <section className="py-20 luxury-gradient">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 50 }}
            whileInView={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8 }}
            viewport={{ once: true }}
            className="text-center mb-16"
          >
            <h2 className="text-4xl md:text-5xl font-luxury font-bold text-luxury-midnight-black mb-6">
              會員方案
            </h2>
            <p className="text-xl text-luxury-midnight-black/80 max-w-3xl mx-auto">
              選擇最適合您的會員等級，享受專屬的尊榮服務與社交體驗
            </p>
          </motion.div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            {membershipTiers.map((tier, index) => (
              <motion.div
                key={tier.name}
                initial={{ opacity: 0, y: 50 }}
                whileInView={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.8, delay: index * 0.2 }}
                viewport={{ once: true }}
                className={`relative p-8 rounded-2xl transition-all duration-300 hover:scale-105 ${
                  tier.popular
                    ? 'bg-luxury-gold text-luxury-midnight-black shadow-2xl'
                    : 'luxury-glass hover:bg-white/10'
                }`}
              >
                {tier.popular && (
                  <div className="absolute -top-4 left-1/2 transform -translate-x-1/2">
                    <div className="bg-luxury-midnight-black text-luxury-gold px-4 py-2 rounded-full text-sm font-medium flex items-center">
                      <Star className="h-4 w-4 mr-1" />
                      最受歡迎
                    </div>
                  </div>
                )}

                <div className="text-center mb-8">
                  <h3 className="text-2xl font-luxury font-bold mb-2">
                    {tier.name}
                  </h3>
                  <div className="flex items-baseline justify-center">
                    <span className="text-4xl font-bold">{tier.price}</span>
                    <span className="text-lg ml-1">{tier.period}</span>
                  </div>
                </div>

                <ul className="space-y-4 mb-8">
                  {tier.features.map((feature, featureIndex) => (
                    <li key={featureIndex} className="flex items-center">
                      <div className={`w-2 h-2 rounded-full mr-3 ${
                        tier.popular ? 'bg-luxury-midnight-black' : 'bg-luxury-gold'
                      }`}></div>
                      <span className="text-sm">{feature}</span>
                    </li>
                  ))}
                </ul>

                <button className={`w-full py-3 px-6 rounded-lg font-medium transition-all duration-300 flex items-center justify-center ${
                  tier.popular
                    ? 'bg-luxury-midnight-black text-luxury-gold hover:bg-luxury-midnight-black/90'
                    : 'luxury-button-outline'
                }`}>
                  {tier.name === 'Black Card' ? '申請邀請' : '立即申請'}
                  <ArrowRight className="h-4 w-4 ml-2" />
                </button>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      <section className="py-20 bg-luxury-midnight-black">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <motion.div
            initial={{ opacity: 0, y: 50 }}
            whileInView={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8 }}
            viewport={{ once: true }}
          >
            <h2 className="text-4xl md:text-5xl font-luxury font-bold text-luxury-gold mb-6">
              準備開始您的尊榮社交之旅？
            </h2>
            <p className="text-xl text-luxury-platinum/80 mb-8 max-w-3xl mx-auto">
              加入台灣最頂級的社交圈，與成功人士建立深度連結，開拓無限可能
            </p>
            <Link to="/register" className="luxury-button text-lg px-12 py-4 inline-flex items-center">
              立即申請加入
              <ArrowRight className="h-5 w-5 ml-2" />
            </Link>
          </motion.div>
        </div>
      </section>
    </div>
  )
}

export default HomePage