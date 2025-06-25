import { useState } from 'react'
import { motion } from 'framer-motion'
import { Crown, Lock, Star, Calendar, Users, Diamond, Shield, Zap } from 'lucide-react'

const VVIPPage = () => {
  const [selectedCategory, setSelectedCategory] = useState('all')

  const vvipEvents = [
    {
      id: '1',
      name: '私人島嶼奢華假期',
      description: '專屬包島體驗，米其林主廚隨行，私人管家服務',
      dateTime: '2024-12-20T10:00:00',
      location: '馬爾地夫私人島嶼',
      price: 500000,
      capacity: 8,
      currentAttendees: 5,
      category: 'travel',
      image: '/api/placeholder/600/400',
      exclusivityLevel: 'VVIP',
      tags: ['包島', '米其林', '私人管家', '奢華'],
      amenities: ['私人飛機接送', '24小時管家服務', '米其林主廚', '水上活動', '私人海灘']
    },
    {
      id: '2',
      name: '名人主廚私宴',
      description: '世界級名廚親自料理，限定6位賓客的私密晚宴',
      dateTime: '2024-12-15T19:00:00',
      location: '私人莊園',
      price: 80000,
      capacity: 6,
      currentAttendees: 4,
      category: 'dining',
      image: '/api/placeholder/600/400',
      exclusivityLevel: 'VVIP',
      tags: ['名廚', '私宴', '限定', '獨家'],
      amenities: ['世界級主廚', '私人莊園', '頂級食材', '個人定制菜單', '私人侍酒師']
    },
    {
      id: '3',
      name: '藝術品私人拍賣會',
      description: '國際知名拍賣行舉辦的私人預展與拍賣',
      dateTime: '2024-12-22T15:00:00',
      location: '私人藝廊',
      price: 50000,
      capacity: 12,
      currentAttendees: 8,
      category: 'art',
      image: '/api/placeholder/600/400',
      exclusivityLevel: 'VVIP',
      tags: ['拍賣', '藝術品', '收藏', '投資'],
      amenities: ['私人預展', '專業估價師', '拍賣優先權', '收藏顧問', '運輸保險']
    }
  ]

  const vvipPerks = [
    {
      icon: Crown,
      title: '專屬禮賓服務',
      description: '24/7 專人服務，滿足您的每一個需求'
    },
    {
      icon: Diamond,
      title: '獨家活動優先權',
      description: '搶先參與最頂級、最獨特的社交活動'
    },
    {
      icon: Shield,
      title: '最高隱私保障',
      description: '軍用級加密技術，絕對保護您的身份與隱私'
    },
    {
      icon: Zap,
      title: '客製化體驗',
      description: '根據您的喜好量身打造專屬活動與服務'
    }
  ]

  const membershipStats = {
    totalMembers: 156,
    activeEvents: 12,
    exclusiveVenues: 45,
    globalPartners: 28
  }

  const categories = [
    { id: 'all', name: '全部活動', count: vvipEvents.length },
    { id: 'dining', name: '頂級餐飲', count: 1 },
    { id: 'travel', name: '奢華旅遊', count: 1 },
    { id: 'art', name: '藝術收藏', count: 1 },
    { id: 'business', name: '商務社交', count: 0 }
  ]

  const filteredEvents = selectedCategory === 'all' 
    ? vvipEvents 
    : vvipEvents.filter(event => event.category === selectedCategory)

  const formatDate = (dateString: string) => {
    const date = new Date(dateString)
    return date.toLocaleDateString('zh-TW', {
      month: 'long',
      day: 'numeric',
      weekday: 'long'
    })
  }

  const formatTime = (dateString: string) => {
    const date = new Date(dateString)
    return date.toLocaleTimeString('zh-TW', {
      hour: '2-digit',
      minute: '2-digit'
    })
  }

  return (
    <div className="min-h-screen bg-luxury-midnight-black">
      {/* Hero Section */}
      <section className="relative py-20 overflow-hidden">
        <div className="absolute inset-0 luxury-gradient opacity-20"></div>
        <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <motion.div
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8 }}
          >
            <div className="inline-flex items-center justify-center w-16 h-16 bg-luxury-gold rounded-full mb-6">
              <Crown className="h-8 w-8 text-luxury-midnight-black" />
            </div>
            <h1 className="text-5xl md:text-7xl font-luxury font-bold text-luxury-gold mb-6">
              VVIP 專區
            </h1>
            <p className="text-xl md:text-2xl text-luxury-platinum/80 max-w-4xl mx-auto leading-relaxed">
              專為最頂級會員打造的獨家體驗空間
              <br />
              享受前所未有的奢華與尊榮
            </p>
          </motion.div>
        </div>
      </section>

      {/* Stats Section */}
      <section className="py-16 bg-luxury-midnight-black/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 30 }}
            whileInView={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8 }}
            viewport={{ once: true }}
            className="grid grid-cols-2 md:grid-cols-4 gap-8"
          >
            <div className="text-center">
              <div className="text-4xl font-bold text-luxury-gold mb-2">
                {membershipStats.totalMembers}
              </div>
              <div className="text-luxury-platinum/80">VVIP 會員</div>
            </div>
            <div className="text-center">
              <div className="text-4xl font-bold text-luxury-gold mb-2">
                {membershipStats.activeEvents}
              </div>
              <div className="text-luxury-platinum/80">獨家活動</div>
            </div>
            <div className="text-center">
              <div className="text-4xl font-bold text-luxury-gold mb-2">
                {membershipStats.exclusiveVenues}
              </div>
              <div className="text-luxury-platinum/80">專屬場地</div>
            </div>
            <div className="text-center">
              <div className="text-4xl font-bold text-luxury-gold mb-2">
                {membershipStats.globalPartners}
              </div>
              <div className="text-luxury-platinum/80">全球夥伴</div>
            </div>
          </motion.div>
        </div>
      </section>

      {/* VVIP Perks */}
      <section className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 30 }}
            whileInView={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8 }}
            viewport={{ once: true }}
            className="text-center mb-16"
          >
            <h2 className="text-4xl md:text-5xl font-luxury font-bold text-luxury-gold mb-6">
              VVIP 專屬特權
            </h2>
            <p className="text-xl text-luxury-platinum/80 max-w-3xl mx-auto">
              超越一般會員的頂級服務體驗
            </p>
          </motion.div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8">
            {vvipPerks.map((perk, index) => (
              <motion.div
                key={perk.title}
                initial={{ opacity: 0, y: 30 }}
                whileInView={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.8, delay: index * 0.1 }}
                viewport={{ once: true }}
                className="luxury-glass p-8 rounded-2xl text-center hover:bg-luxury-gold/5 transition-all duration-300 group"
              >
                <perk.icon className="h-12 w-12 text-luxury-gold mx-auto mb-6 group-hover:scale-110 transition-transform duration-300" />
                <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-4">
                  {perk.title}
                </h3>
                <p className="text-luxury-platinum/80 leading-relaxed">
                  {perk.description}
                </p>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* VVIP Events */}
      <section className="py-20 bg-luxury-midnight-black/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 30 }}
            whileInView={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8 }}
            viewport={{ once: true }}
            className="text-center mb-12"
          >
            <h2 className="text-4xl md:text-5xl font-luxury font-bold text-luxury-gold mb-6">
              獨家活動
            </h2>
            <p className="text-xl text-luxury-platinum/80 max-w-3xl mx-auto">
              僅限 VVIP 會員參與的頂級社交體驗
            </p>
          </motion.div>

          {/* Category Filter */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.2 }}
            viewport={{ once: true }}
            className="flex flex-wrap justify-center gap-4 mb-12"
          >
            {categories.map(category => (
              <button
                key={category.id}
                onClick={() => setSelectedCategory(category.id)}
                className={`px-6 py-3 rounded-lg font-medium transition-all duration-300 ${
                  selectedCategory === category.id
                    ? 'bg-luxury-gold text-luxury-midnight-black'
                    : 'bg-luxury-gold/20 text-luxury-gold hover:bg-luxury-gold/30'
                }`}
              >
                {category.name} ({category.count})
              </button>
            ))}
          </motion.div>

          {/* Events Grid */}
          <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-8">
            {filteredEvents.map((event, index) => (
              <motion.div
                key={event.id}
                initial={{ opacity: 0, y: 30 }}
                whileInView={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.8, delay: index * 0.1 }}
                viewport={{ once: true }}
                className="luxury-glass rounded-2xl overflow-hidden hover:bg-luxury-gold/5 transition-all duration-300 group"
              >
                <div className="relative">
                  <img
                    src={event.image}
                    alt={event.name}
                    className="w-full h-48 object-cover group-hover:scale-105 transition-transform duration-300"
                  />
                  <div className="absolute top-4 left-4">
                    <span className="px-3 py-1 bg-luxury-gold text-luxury-midnight-black text-xs font-medium rounded-full">
                      {event.exclusivityLevel}
                    </span>
                  </div>
                  <div className="absolute top-4 right-4">
                    <div className="w-10 h-10 bg-luxury-midnight-black/70 backdrop-blur-sm rounded-full flex items-center justify-center">
                      <Lock className="h-5 w-5 text-luxury-gold" />
                    </div>
                  </div>
                </div>

                <div className="p-6">
                  {/* Event Title */}
                  <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-3 group-hover:text-luxury-gold/90 transition-colors">
                    {event.name}
                  </h3>
                  
                  {/* Event Description */}
                  <p className="text-luxury-platinum/80 text-sm mb-4 line-clamp-2 leading-relaxed">
                    {event.description}
                  </p>

                  {/* Event Details */}
                  <div className="space-y-2 mb-4">
                    <div className="flex items-center text-luxury-platinum/70 text-sm">
                      <Calendar className="h-4 w-4 mr-2 text-luxury-gold flex-shrink-0" />
                      <span>{formatDate(event.dateTime)} {formatTime(event.dateTime)}</span>
                    </div>
                    <div className="flex items-center text-luxury-platinum/70 text-sm">
                      <Users className="h-4 w-4 mr-2 text-luxury-gold flex-shrink-0" />
                      <span>{event.currentAttendees}/{event.capacity} 人</span>
                    </div>
                  </div>

                  {/* Event Tags */}
                  <div className="flex flex-wrap gap-2 mb-6">
                    {event.tags.slice(0, 4).map((tag, tagIndex) => (
                      <span
                        key={tagIndex}
                        className="px-3 py-1 bg-luxury-gold/20 text-luxury-gold text-xs rounded-full font-medium"
                      >
                        {tag}
                      </span>
                    ))}
                  </div>

                  {/* Price and Action */}
                  <div className="flex items-center justify-between pt-4 border-t border-luxury-gold/20">
                    <div className="text-2xl font-bold text-luxury-gold">
                      NT$ {event.price.toLocaleString()}
                    </div>
                    <button className="px-6 py-2 bg-luxury-gold text-luxury-midnight-black rounded-lg hover:bg-luxury-gold/90 transition-all duration-300 text-sm font-semibold shadow-lg hover:shadow-luxury-gold/25">
                      查看詳情
                    </button>
                  </div>
                </div>
              </motion.div>
            ))}
          </div>

          {filteredEvents.length === 0 && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              className="text-center py-12"
            >
              <Lock className="h-16 w-16 text-luxury-gold/50 mx-auto mb-4" />
              <h3 className="text-xl font-luxury text-luxury-gold mb-2">
                暫無此類別活動
              </h3>
              <p className="text-luxury-platinum/60">
                請選擇其他類別或稍後再來查看
              </p>
            </motion.div>
          )}
        </div>
      </section>

      {/* Exclusive Access CTA */}
      <section className="py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <motion.div
            initial={{ opacity: 0, y: 30 }}
            whileInView={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8 }}
            viewport={{ once: true }}
            className="luxury-glass p-12 rounded-2xl text-center"
          >
            <Crown className="h-16 w-16 text-luxury-gold mx-auto mb-6" />
            <h2 className="text-4xl font-luxury font-bold text-luxury-gold mb-6">
              申請 VVIP 會員資格
            </h2>
            <p className="text-xl text-luxury-platinum/80 mb-8 max-w-3xl mx-auto">
              VVIP 會員採邀請制，需經過嚴格審核。
              <br />
              如您符合資格，我們的專屬顧問將與您聯繫。
            </p>
            
            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <button className="luxury-button px-8 py-4 text-lg">
                申請邀請函
              </button>
              <button className="luxury-button-outline px-8 py-4 text-lg">
                聯繫專屬顧問
              </button>
            </div>

            <div className="mt-8 text-luxury-platinum/60 text-sm">
              <p>* VVIP 會員需滿足特定財富與社會地位要求</p>
              <p>* 年費 NT$ 500,000 起，享受全年無限制專屬服務</p>
            </div>
          </motion.div>
        </div>
      </section>
    </div>
  )
}

export default VVIPPage