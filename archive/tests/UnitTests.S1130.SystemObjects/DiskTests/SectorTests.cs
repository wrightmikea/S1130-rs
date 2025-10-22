using S1130.SystemObjects.Devices.Disks;
using Xunit;

namespace UnitTests.S1130.SystemObjects.DeviceTests.DiskTests
{
    public class DiskSectorTests
    {
        [Fact]
        public void SectorIs321WordsTest()
        {
            var sector = new DiskSector();
            Assert.Equal(321, sector.Data.Length);
        }

        [Fact]
        public void LoadTest()
        {
            var sector = new DiskSector();
            ushort[] data = new ushort[321];
            for (ushort i = 0; i < data.Length; i++)
            {
                data[i] = i;
            }
            sector.Load(data);
            Assert.Equal(data, sector.Data);
        }

        [Fact]
        public void LoadMustBeUshort321()
        {
            var notsector =  new ushort[2];
           // Assert.Throws<>
        
        //When
        
        //Then
        }
    }
}